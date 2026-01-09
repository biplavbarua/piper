use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use anyhow::Result;

pub struct CompressionStats {
    pub original_size: u64,
    pub compressed_size: u64,
    pub output_path: PathBuf,
}

pub fn compress_file(input_path: &Path) -> Result<CompressionStats> {
    let input_file = File::open(input_path)?;
    let original_size = input_file.metadata()?.len();
    let reader = BufReader::new(input_file);

    let output_path = input_path.with_extension(format!("{}.zst", input_path.extension().unwrap_or_default().to_string_lossy()));
    let output_file = File::create(&output_path)?;
    let writer = BufWriter::new(output_file);

    // Pied Piper "Middle-Out" Level (15 for high compression)
    zstd::stream::copy_encode(reader, writer, 15)?;

    let compressed_size = output_path.metadata()?.len();

    Ok(CompressionStats {
        original_size,
        compressed_size,
        output_path,
    })
}
