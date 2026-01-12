use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

// Mock the compressor module content here for isolation
fn compress_file(input_path: &std::path::Path, level: i32) -> anyhow::Result<()> {
    println!("DEBUG: Starting compression for {:?}", input_path);
    let metadata = input_path.metadata()?;
    
    if metadata.is_dir() {
        println!("DEBUG: Identified as directory.");
        compress_directory(input_path, level)
    } else {
        println!("DEBUG: Identified as file.");
        Ok(())
    }
}

fn compress_directory(input_path: &std::path::Path, level: i32) -> anyhow::Result<()> {
    let original_size = 1000; // Fake it
    println!("DEBUG: compress_directory called on {:?}", input_path);

    let output_path = PathBuf::from(format!("{}.tar.zst", input_path.to_string_lossy()));
    let temp_path = output_path.with_extension("tmp");
    println!("DEBUG: Output path: {:?}", output_path);
    println!("DEBUG: Temp path: {:?}", temp_path);

    let file = File::create(&temp_path)?;
    println!("DEBUG: Temp file created.");
    
    // Create a dummy tar zst
    let encoder = zstd::stream::write::Encoder::new(file, level)?;
    let mut tar = tar::Builder::new(encoder);
    
    let dirname = input_path.file_name().unwrap();
    println!("DEBUG: Appending dir {:?} with name {:?}", input_path, dirname);
    tar.append_dir_all(dirname, input_path)?;
    
    let encoder = tar.into_inner()?;
    encoder.finish()?;
    println!("DEBUG: Archive finished.");

    // Check size
    let compressed_size = temp_path.metadata()?.len();
    println!("DEBUG: Compressed size: {}", compressed_size);

    // Force commit
    println!("DEBUG: Renaming temp to output...");
    std::fs::rename(&temp_path, &output_path)?;
    println!("DEBUG: Success rename. Removing original...");
    std::fs::remove_dir_all(input_path)?;
    println!("DEBUG: Success remove.");

    Ok(())
}

fn main() {
    let home = dirs::home_dir().unwrap();
    let test_dir = home.join("piper_test_ground/my_project/node_modules");

    println!("Running Debug on: {:?}", test_dir);

    // Create if not exists for test
    if !test_dir.exists() {
         std::fs::create_dir_all(&test_dir).unwrap();
         std::fs::write(test_dir.join("test.txt"), "hello").unwrap();
    }

    match compress_file(&test_dir, 1) {
        Ok(_) => println!("COMPRESSION SUCCESS"),
        Err(e) => println!("COMPRESSION ERROR: {:?}", e),
    }
}
