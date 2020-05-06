use std::env;
use std::error;
use std::fs;

fn main() -> Result<(), Box<dyn error::Error>> {
    let home = env::var("HOME")?;

    fs::copy(format!("{}{}", home, "/load_connection_string.rs"), "./connection-string.fn")?;
    Ok(())
}
