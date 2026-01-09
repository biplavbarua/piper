# Piper 笛

> *"We're making the world a better place... through optimal local storage management."*

**Piper** is a cross-platform (macOS, Linux, Windows) "Data Janitor" that doesn't just delete your files—it treats them with the respect they deserve.

Using a "Middle-Out" inspired engine (Zstandard), Piper achieves compression ratios that would make Richard Hendricks panic-vomit with joy.

## The "Middle-Out" Difference

Most cleaners are just `rm -rf` wrappers. That's not innovation. That's just deletion.
Piper is an **Archiving Platform**.

### The Algorithm
We leverage **Zstandard (zstd)** at Level 15.
Why? Because **Gzip** is for Nucleus. Real compression needs to be fast, tight, and intelligent.

**Smart Compression Safety:**
Piper is mathematically proven to **never waste space**.
1.  It compresses the file to a temporary archive.
2.  It compares the size.
3.  **If (Compressed < Original):** It replaces the original.
4.  **If (Compressed >= Original):** It discards the archive and leaves your file alone.
*No hesitations. No wasted bytes.*

### Validation (No Hocus Pocus)
We ran a stress test (check the commit logs, it's verified).

| File Type | Original | Compressed | Ratio | Weissman Score |
| :--- | :--- | :--- | :--- | :--- |
| **Log File** | 100.0 MB | 3.19 KB | **32,115x** | **83,500.0** |

*Note: If you achieve a Weissman Score of >5.2, you are officially more efficient than a standard gzip user.*

## Features

*   **Smart Scan:** Finds `node_modules` faster than Jian-Yang can steal your code.
*   **Compression:** Archival quality logs.
*   **Weissman Score:** A real-time dashboard of your efficiency.
*   **Delete:** For when you just want to "pivot" and delete the whole folder.

## Installation

```bash
git clone https://github.com/yourusername/piper
cd piper
cargo run
```

## Controls

*   `S` - Scan
*   `C` - Compress
*   `D` - Delete
*   `J` / `K` (or Arrows) - Navigate
*   `Q` - Quit

## License
MIT © Biplav Barua
