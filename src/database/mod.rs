use std::fs::File;
use std::path::Path;

pub fn create_database(name: &str) -> Result<(), std::io::Error> {
    println!("Creating database: {}", name);
    let path = Path::new("db").join(name);

    // Create the file at the specified path
    File::create(path)?;
    Ok(())
}