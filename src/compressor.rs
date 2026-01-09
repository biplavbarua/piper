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

    // Robustness Check: Only replace if we actually save space
    if compressed_size < original_size {
        std::fs::remove_file(input_path)?;
        Ok(CompressionStats {
            original_size,
            compressed_size,
            output_path: output_path.clone(),
        })
    } else {
        // Compression failed to save space (e.g. already compressed file).
        // Remove the larger .zst file and return unmodified stats.
        std::fs::remove_file(&output_path)?;
        Ok(CompressionStats {
            original_size,
            compressed_size: original_size, // No savings
            output_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_compress_saves_space() -> Result<()> {
        // Setup: Create compressible file
        let path = PathBuf::from("test_compressible.log");
        let mut file = File::create(&path)?;
        // Write 1MB of 'A' (highly compressible)
        for _ in 0..1024 {
            file.write_all(&[b'A'; 1024])?;
        }

        let original_size = path.metadata()?.len();
        
        // Act
        let stats = compress_file(&path)?;

        // Assert
        assert!(stats.compressed_size < original_size);
        assert!(!path.exists(), "Original file should be deleted");
        assert!(path.with_extension("log.zst").exists(), "Compressed file should exist");

        // Cleanup
        std::fs::remove_file(path.with_extension("log.zst"))?;
        Ok(())
    }

    #[test]
    fn test_compress_skips_bad_ratio() -> Result<()> {
        // Setup: Create incompressible file (random data)
        // Note: In real life randomness is hard to compress. 
        // We'll simulate by creating a small file where header overhead > savings
        let path = PathBuf::from("test_tiny.log");
        let mut file = File::create(&path)?;
        file.write_all(b"random")?; // Too small to save space with zstd headers

        let original_size = path.metadata()?.len();

        // Act
        let stats = compress_file(&path)?;

        // Assert
        assert_eq!(stats.compressed_size, original_size, "Should report original size if skipped");
        assert!(path.exists(), "Original file should STILL exist");
        assert!(!path.with_extension("log.zst").exists(), "Compressed file should NOT exist");

        // Cleanup
        std::fs::remove_file(path)?;
        Ok(())
    }
}
