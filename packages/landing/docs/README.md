# neuro.lab documentation

**neuro.lab** is a local-first desktop workspace for EEG, MEG, and MRI research
data. The React interface runs inside Tauri and calls typed Rust commands
directly — no server dependency. All computation and persistence stay on the
local filesystem.

## Sections

- **Getting started** — install the desktop app and use the `neuro` CLI.
- **Concepts** — pipelines, datasets, and provenance.
- **Guides** — task-oriented walkthroughs and the MCP server.

## What it does

- Inspect, validate, and process EEG, MEG, and MRI recordings in one workspace.
- Reproducible JSON pipeline chains for filtering, artifact rejection,
  epoching, and more.
- Versioned datasets, experiments, analysis runs, and artifacts with
  provenance.
- `neuro` CLI for scripting: inspect, validate, convert, and export — plus
  `neuro mcp` for MCP clients.
- Runs fully offline; data never leaves your machine.
