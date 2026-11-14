mod mcp;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "neuro",
    version,
    about = "Neuro Lab command-line interface",
    long_about = "Local EEG/MEG/MRI inspection, validation, conversion, and export.\nAll output is JSON on stdout; diagnostics go to stderr."
)]
struct Cli {
    /// Pretty-print JSON output
    #[arg(long, global = true)]
    pretty: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Inspect an EEG recording (NPY, EDF/EDF+, BrainVision)
    InspectEeg {
        path: PathBuf,
        /// Fallback sampling rate for formats without one (NPY)
        #[arg(long)]
        sfreq: Option<f64>,
    },
    /// Validate an EEG file set and report findings
    ValidateEeg { path: PathBuf },
    /// Generate a deterministic synthetic EEG recording (NPY + manifest)
    GenerateEeg {
        directory: PathBuf,
        #[arg(long, default_value = "synthetic")]
        name: String,
        #[arg(long, default_value_t = 16)]
        channels: usize,
        #[arg(long, default_value_t = 10_000)]
        samples: usize,
        #[arg(long, default_value_t = 250.0)]
        sfreq: f64,
        #[arg(long, default_value_t = 42)]
        seed: u64,
    },
    /// Export an EEG recording to EDF+
    ExportEeg {
        path: PathBuf,
        /// Output path (default: <stem>.export.edf beside the input)
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long)]
        sfreq: Option<f64>,
    },
    /// Inspect a MEG recording (FIF, CTF .ds, KIT, BIDS, OPM)
    InspectMeg { path: PathBuf },
    /// Validate a MEG file set and report findings
    ValidateMeg { path: PathBuf },
    /// Inspect a NIfTI volume
    InspectMri { path: PathBuf },
    /// Convert a DICOM file or series directory to NIfTI
    ConvertDicom {
        input: PathBuf,
        /// Output .nii path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Run an MCP server over stdio JSON-RPC
    Mcp,
}

fn print_json(value: &serde_json::Value, pretty: bool) {
    let rendered = if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    };
    match rendered {
        Ok(text) => println!("{text}"),
        Err(error) => {
            eprintln!("failed to serialize output: {error}");
            std::process::exit(1);
        }
    }
}

fn fail(message: impl std::fmt::Display) -> ! {
    eprintln!("error: {message}");
    std::process::exit(1);
}

fn main() {
    let cli = Cli::parse();
    if matches!(cli.command, Command::Mcp) {
        if let Err(error) = mcp::run() {
            fail(format!("mcp: {error}"));
        }
        return;
    }
    let result = match cli.command {
        Command::InspectEeg { path, sfreq } => {
            resolve_sfreq(&path, sfreq).and_then(|rate| inspect_eeg(&path, rate))
        }
        Command::ValidateEeg { path } => validate_eeg(&path),
        Command::GenerateEeg {
            directory,
            name,
            channels,
            samples,
            sfreq,
            seed,
        } => {
            neuro_eeg::write_synthetic_recording(&directory, &name, channels, samples, sfreq, seed)
                .map(|value| serde_json::json!({ "result": value }))
                .map_err(|error| error.to_string())
        }
        Command::ExportEeg {
            path,
            output,
            sfreq,
        } => resolve_sfreq(&path, sfreq).and_then(|rate| export_eeg(&path, output, rate)),
        Command::InspectMeg { path } => neuro_meg::inspect_recording(&path)
            .map(|metadata| serde_json::to_value(&metadata).unwrap_or_default())
            .map_err(|error| error.to_string()),
        Command::ValidateMeg { path } => {
            neuro_meg::validate_file_set(&path).map_err(|error| error.to_string())
        }
        Command::InspectMri { path } => neuro_mri::inspect_nifti(&path)
            .map(|info| serde_json::to_value(&info).unwrap_or_default())
            .map_err(|error| error.to_string()),
        Command::ConvertDicom { input, output } => if input.is_dir() {
            neuro_mri::dicom::dicom_series_to_nifti(&input, &output)
        } else {
            neuro_mri::dicom::dicom_to_nifti(&input, &output)
        }
        .map_err(|error| error.to_string()),
        Command::Mcp => unreachable!("handled before dispatch"),
    };
    match result {
        Ok(value) => print_json(&value, cli.pretty),
        Err(message) => fail(message),
    }
}

fn inspect_eeg(path: &PathBuf, fallback_sfreq: f64) -> Result<serde_json::Value, String> {
    let (matrix, names, _units, sfreq, events) =
        neuro_eeg::load_recording_with_events(path, fallback_sfreq).map_err(|e| e.to_string())?;
    let samples = matrix.first().map_or(0, Vec::len);
    Ok(serde_json::json!({
        "format": detect_eeg_format(path),
        "channels": names.len(),
        "channel_names": names,
        "samples": samples,
        "sampling_frequency_hz": sfreq,
        "duration_seconds": if sfreq > 0.0 { samples as f64 / sfreq } else { 0.0 },
        "events": events,
    }))
}

fn validate_eeg(path: &PathBuf) -> Result<serde_json::Value, String> {
    let (format, findings) = neuro_eeg::validate_recording(path);
    Ok(serde_json::json!({ "format": format, "findings": findings }))
}

fn export_eeg(
    path: &PathBuf,
    output: Option<PathBuf>,
    fallback_sfreq: f64,
) -> Result<serde_json::Value, String> {
    let (matrix, names, units, sfreq) =
        neuro_eeg::load_recording(path, fallback_sfreq).map_err(|e| e.to_string())?;
    let output = output.unwrap_or_else(|| {
        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("recording");
        path.with_file_name(format!("{stem}.export.edf"))
    });
    neuro_eeg::edf::write_edf(&output, &matrix, &names, &units, sfreq)
        .map_err(|error| error.to_string())?;
    Ok(serde_json::json!({ "output_path": output.to_string_lossy() }))
}

/// Resolve the sampling rate for formats that don't embed one (NPY):
/// explicit `--sfreq` wins, then the `<name>.json` channel manifest.
/// Non-NPY formats carry their own rate, so any positive value works.
fn resolve_sfreq(path: &PathBuf, explicit: Option<f64>) -> Result<f64, String> {
    if let Some(value) = explicit {
        if value <= 0.0 || !value.is_finite() {
            return Err("--sfreq must be a positive finite number".into());
        }
        return Ok(value);
    }
    if let Some(rate) = neuro_eeg::channel_manifest(path)
        .sampling_frequency_hz
        .filter(|value| *value > 0.0 && value.is_finite())
    {
        return Ok(rate);
    }
    let is_npy = path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("npy"));
    if is_npy {
        Err(".npy files have no embedded rate: pass --sfreq or provide a <name>.json channel manifest".into())
    } else {
        Ok(1.0)
    }
}

fn detect_eeg_format(path: &PathBuf) -> String {
    if neuro_eeg::brainvision::vhdr_path(path).exists() {
        return "brainvision".into();
    }
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "edf" | "edf+" => "edf".into(),
        "npy" => "npy".into(),
        _ => "unknown".into(),
    }
}
