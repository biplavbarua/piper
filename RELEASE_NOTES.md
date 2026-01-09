# Piper v1.0.0: The "Middle-Out" Release 🚀

**The Middle-Out Data Optimizer is finally here.**

Piper is a high-performance, terminal-based tool designed to aggressively scan your `~/Developer` directory for large, compressible artifacts (logs, text files, etc.) and shrink them down using robust zstd compression—without deleting anything unless it guarantees space savings.

## 🌟 Key Features

### 🏎️ Parallel Compression Engine
- Powered by `rayon`, Piper now utilizes **all available CPU cores** to compress files simultaneously.
- "Hesitation-free" performance: the UI remains buttery smooth while your CPU burns through thousands of files.

### 🛡️ Smart Compression Safety
- **Atomic Replacement**: Piper creates a `.zst` candidate first.
- **Verification**: It compares the original vs. compressed size.
- **Safety Guarantee**: The original file is ONLY replaced if the compressed version is smaller. Zero risk of data bloat.

### 📟 Hacker-Aesthetic TUI
- **Live Status**: Real-time spinners (`⠋ ⠙ ⠹ ⠸`) and progress bars.
- **Details Mode**: Press `<Enter>` on any file to inspect exact byte-level savings.
- **Weissman Score**: A dynamic efficiency metric shown in the header.

### ⌨️ Controls
- `[S]` **Scan**: Scan `~/Developer` for compressible artifacts.
- `[C]` **Compress**: Ignite the parallel compression engine.
- `[D]` **Delete**: Manually nuke files you don't need.
- `[Enter]` **Details**: View compression stats.

## Installation

```bash
cargo install --path .
```

## "Middle-Out"
> "We don't deleting data. We optimize it."
