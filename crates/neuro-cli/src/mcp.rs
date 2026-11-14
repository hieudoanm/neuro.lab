//! MCP server over stdio JSON-RPC.
//!
//! Tools delegate to the `neuro` binary's own subcommands so MCP behaviour
//! always matches the CLI. Configure an MCP client with: `neuro mcp`

use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::process::Command;

const SERVER_INFO: &str = "neuro-mcp";
const PROTOCOL_VERSION: &str = "2024-11-05";

fn arg(params: &Value, key: &str) -> Option<String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .filter(|s| !s.is_empty())
}

fn num(params: &Value, key: &str) -> Option<String> {
    params.get(key).and_then(|v| {
        v.as_u64()
            .map(|n| n.to_string())
            .or_else(|| v.as_i64().map(|n| n.to_string()))
            .or_else(|| v.as_f64().map(|n| n.to_string()))
    })
}

fn run_cli(args: &[String]) -> Value {
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(err) => {
            return json!({
                "content": [{"type": "text", "text": format!("cannot locate neuro binary: {err}")}],
                "isError": true
            })
        }
    };
    match Command::new(exe).args(args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if output.status.success() {
                let text = if stdout.is_empty() { stderr } else { stdout };
                json!({"content": [{"type": "text", "text": text}]})
            } else {
                json!({
                    "content": [{"type": "text", "text": stderr}],
                    "isError": true
                })
            }
        }
        Err(err) => json!({
            "content": [{"type": "text", "text": format!("failed to run neuro: {err}")}],
            "isError": true
        }),
    }
}

fn build_args(name: &str, params: &Value) -> Option<Vec<String>> {
    let mut args = vec![name.replace('_', "-")];
    match name {
        "inspect_eeg" | "validate_eeg" | "inspect_meg" | "validate_meg" | "inspect_mri" => {
            args.push(arg(params, "path")?);
            if name == "inspect_eeg" {
                if let Some(v) = num(params, "sfreq") {
                    args.extend(["--sfreq".to_string(), v]);
                }
            }
        }
        "export_eeg" => {
            args.push(arg(params, "path")?);
            if let Some(v) = arg(params, "output") {
                args.extend(["--output".to_string(), v]);
            }
            if let Some(v) = num(params, "sfreq") {
                args.extend(["--sfreq".to_string(), v]);
            }
        }
        "generate_eeg" => {
            args.push(arg(params, "directory")?);
            for (key, flag) in [
                ("name", "--name"),
                ("channels", "--channels"),
                ("samples", "--samples"),
                ("sfreq", "--sfreq"),
                ("seed", "--seed"),
            ] {
                if let Some(v) = arg(params, key).or_else(|| num(params, key)) {
                    args.extend([flag.to_string(), v]);
                }
            }
        }
        "convert_dicom" => {
            args.push(arg(params, "input")?);
            args.extend(["--output".to_string(), arg(params, "output")?]);
        }
        _ => return None,
    }
    Some(args)
}

fn path_schema(extra: Value) -> Value {
    let mut properties = json!({
        "path": {"type": "string", "description": "Path to the recording file or file set"}
    });
    if let (Some(base), Some(extra)) = (properties.as_object_mut(), extra.as_object()) {
        for (key, value) in extra {
            base.insert(key.clone(), value.clone());
        }
    }
    json!({
        "type": "object",
        "properties": properties,
        "required": ["path"],
        "additionalProperties": false
    })
}

fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "inspect_eeg",
                "description": "Inspect an EEG recording (NPY, EDF/EDF+, BrainVision) and return JSON metadata",
                "inputSchema": path_schema(json!({
                    "sfreq": {"type": "number", "description": "Fallback sampling rate for formats without one (NPY)"}
                }))
            },
            {
                "name": "validate_eeg",
                "description": "Validate an EEG file set and report findings",
                "inputSchema": path_schema(json!({}))
            },
            {
                "name": "export_eeg",
                "description": "Export an EEG recording to EDF+",
                "inputSchema": path_schema(json!({
                    "output": {"type": "string", "description": "Output path (default: <stem>.export.edf beside the input)"},
                    "sfreq": {"type": "number", "description": "Sampling rate override"}
                }))
            },
            {
                "name": "generate_eeg",
                "description": "Generate a deterministic synthetic EEG recording (NPY + manifest)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "directory": {"type": "string", "description": "Output directory"},
                        "name": {"type": "string", "description": "Recording name (default: synthetic)"},
                        "channels": {"type": "integer", "description": "Channel count (default: 16)"},
                        "samples": {"type": "integer", "description": "Sample count (default: 10000)"},
                        "sfreq": {"type": "number", "description": "Sampling rate in Hz (default: 250)"},
                        "seed": {"type": "integer", "description": "Random seed (default: 42)"}
                    },
                    "required": ["directory"],
                    "additionalProperties": false
                }
            },
            {
                "name": "inspect_meg",
                "description": "Inspect a MEG recording (FIF, CTF .ds, KIT, BIDS, OPM)",
                "inputSchema": path_schema(json!({}))
            },
            {
                "name": "validate_meg",
                "description": "Validate a MEG file set and report findings",
                "inputSchema": path_schema(json!({}))
            },
            {
                "name": "inspect_mri",
                "description": "Inspect a NIfTI volume",
                "inputSchema": path_schema(json!({}))
            },
            {
                "name": "convert_dicom",
                "description": "Convert a DICOM file or series directory to NIfTI",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "input": {"type": "string", "description": "DICOM file or series directory"},
                        "output": {"type": "string", "description": "Output .nii path"}
                    },
                    "required": ["input", "output"],
                    "additionalProperties": false
                }
            }
        ]
    })
}

fn handle_request(request: &Value) -> Option<Value> {
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let has_id = request.get("id").is_some();

    let result = match method {
        "initialize" => json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": SERVER_INFO, "version": env!("CARGO_PKG_VERSION")}
        }),
        "ping" => Value::Null,
        "tools/list" => tools_list(),
        "tools/call" => {
            let params = request.get("params").cloned().unwrap_or(Value::Null);
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
            match build_args(name, &arguments) {
                Some(args) => run_cli(&args),
                None => json!({
                    "content": [{"type": "text", "text": format!("Unknown tool or missing required arguments: {name}")}],
                    "isError": true
                }),
            }
        }
        _ => {
            if !has_id {
                return None;
            }
            return Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32601, "message": format!("Method not found: {method}")}
            }));
        }
    };

    if !has_id {
        return None;
    }
    Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

/// Run the MCP server on stdin/stdout until EOF.
pub fn run() -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(trimmed) {
            Ok(request) => {
                if let Some(response) = handle_request(&request) {
                    writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                    stdout.flush()?;
                }
            }
            Err(err) => {
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": {"code": -32700, "message": format!("Parse error: {err}")}
                });
                writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                stdout.flush()?;
            }
        }
    }
    Ok(())
}
