use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use anyhow::Result;

pub struct CompressionStats {
    pub original_size: u64,
    pub compressed_size: u64,
    pub output_path: PathBuf,
}

pub fn compress_file(input_path: &Path, level: i32) -> Result<CompressionStats> {
    let metadata = input_path.metadata()?;
    
    if metadata.is_dir() {
        compress_directory(input_path, level)
    } else {
        compress_single_file(input_path, level, metadata.len())
    }
}

fn compress_single_file(input_path: &Path, level: i32, original_size: u64) -> Result<CompressionStats> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);

    // Atomic Write Pattern: Write to .tmp first
    let output_path = input_path.with_extension(format!("{}.zst", input_path.extension().unwrap_or_default().to_string_lossy()));
    let temp_path = output_path.with_extension("zst.tmp");
    
    let output_file = File::create(&temp_path)?;
    let writer = BufWriter::new(output_file);

    // Pied Piper "Middle-Out" Level (Configurable)
    match zstd::stream::copy_encode(reader, writer, level) {
        Ok(_) => {},
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            return Err(e.into());
        }
    }

    finalize_compression(input_path, &output_path, &temp_path, original_size)
}

fn compress_directory(input_path: &Path, level: i32) -> Result<CompressionStats> {
    // Calculate total size first for stats (recursive)
    let original_size = get_dir_size(input_path);

    let parent = input_path.parent().unwrap_or(Path::new("."));
    let dirname = input_path.file_name().ok_or(anyhow::anyhow!("Invalid directory name"))?;
    
    // Output: folder.tar.zst
    let output_path = input_path.with_extension("tar.zst"); 
    // Just appending .tar.zst to "folder" gives "folder.tar.zst" if path is "folder".
    // Wait, PathBuf::from("folder").with_extension("tar.zst") replaces extension? 
    // No, "folder" has no extension. So it becomes "folder.tar.zst".
    // If path is "folder.v1", it becomes "folder.tar.zst".
    // Let's ensure we preserve the name.
    let output_path = PathBuf::from(format!("{}.tar.zst", input_path.to_string_lossy()));

    let temp_path = output_path.with_extension("tmp");

    let file = File::create(&temp_path)?;
    let encoder = zstd::stream::write::Encoder::new(file, level)?;
    let mut tar = tar::Builder::new(encoder);

    // Append dir recursively
    // We want the archive to contain the directory itself, so when unpacking it creates the directory.
    // append_dir_all("name_in_archive", "path_on_disk")
    tar.append_dir_all(dirname, input_path)?;
    
    // Finish Tar
    let encoder = tar.into_inner()?;
    // Finish Zstd
    encoder.finish()?;

    // Finalize
    // For directories, we use trash::delete or fs::remove_dir_all
    // But finalize_compression checks size savings.

    let compressed_size = temp_path.metadata()?.len();

    if compressed_size < original_size {
         std::fs::rename(&temp_path, &output_path)?;
         // Use trash if available, or remove_dir_all?
         // App usually handles deletion of original checks, wait.
         // In compress_single_file below, I see `std::fs::remove_file(input_path)?`.
         // For directories, we should be careful. `std::fs::remove_dir_all`.
         std::fs::remove_dir_all(input_path)?;
         
         Ok(CompressionStats {
             original_size,
             compressed_size,
             output_path,
         })
    } else {
        let _ = std::fs::remove_file(&temp_path);
        Ok(CompressionStats {
             original_size,
             compressed_size: original_size, 
             output_path: input_path.to_path_buf(),
        })
    }
}

fn finalize_compression(input_path: &Path, output_path: &Path, temp_path: &Path, original_size: u64) -> Result<CompressionStats> {
    let compressed_size = temp_path.metadata()?.len();

    if compressed_size < original_size {
        std::fs::rename(temp_path, output_path)?;
        std::fs::remove_file(input_path)?;
        
        Ok(CompressionStats {
            original_size,
            compressed_size,
            output_path: output_path.to_path_buf(),
        })
    } else {
        let _ = std::fs::remove_file(temp_path);
        Ok(CompressionStats {
            original_size,
            compressed_size: original_size, 
            output_path: input_path.to_path_buf(),
        })
    }
}

fn get_dir_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}


pub fn decompress_file(input_path: &Path) -> Result<u64> {
    let file_name = input_path.file_name().unwrap_or_default().to_string_lossy();

    if file_name.ends_with(".tar.zst") {
        decompress_archive(input_path)
    } else if input_path.extension().map_or(false, |ext| ext == "zst") {
        decompress_single(input_path)
    } else {
        Err(anyhow::anyhow!("File is not a supported archive"))
    }
}

fn decompress_single(input_path: &Path) -> Result<u64> {
     let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);

    let output_path = input_path.with_extension(""); // Removes .zst
    
    let output_file = File::create(&output_path)?;
    let writer = BufWriter::new(output_file);

    zstd::stream::copy_decode(reader, writer)?;

    let restored_size = output_path.metadata()?.len();
    std::fs::remove_file(input_path)?;
    
    Ok(restored_size)
}

fn decompress_archive(input_path: &Path) -> Result<u64> {
    let file = File::open(input_path)?;
    let decoder = zstd::stream::read::Decoder::new(file)?;
    let mut archive = tar::Archive::new(decoder);

    // Unpack to parent directory
    let parent = input_path.parent().unwrap_or(Path::new("."));
    archive.unpack(parent)?;

    // We can't easily get strict restored size without calculation, 
    // but we can assume success if unpack didn't fail.
    // Let's try to calculate size of what we just unpacked?
    // It's a directory. The dirname should be what was inside.
    // Usually input is name.tar.zst -> name.
    
    let folder_name = input_path.file_stem().map(|s| {
         // remove .tar from .tar.zst stem?
         // file_stem of 'foo.tar.zst' is 'foo.tar'.
         Path::new(s).file_stem().unwrap_or(s)
    }).unwrap_or_default();
    
    let restored_path = parent.join(folder_name);
    let restored_size = get_dir_size(&restored_path); // Approximation
    
    std::fs::remove_file(input_path)?;

    Ok(restored_size)
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
        let stats = compress_file(&path, 15)?;

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
        let stats = compress_file(&path, 15)?;

        // Assert
        assert_eq!(stats.compressed_size, original_size, "Should report original size if skipped");
        assert!(path.exists(), "Original file should STILL exist");
        assert!(!path.with_extension("log.zst").exists(), "Compressed file should NOT exist");

        // Cleanup
        std::fs::remove_file(path)?;
        Ok(())
    }
}
