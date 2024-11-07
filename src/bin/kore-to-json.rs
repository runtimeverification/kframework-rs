use std::error::Error;
use std::fs::read_to_string;
use std::path::PathBuf;

use clap::Parser;

use kframework::kore;

#[derive(Parser)]
/// Parse a textual KORE pattern from a file and convert it to JSON
struct Args {
    /// Path of .kore file to parse
    path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let text = read_to_string(args.path)?;
    let mut parser = kore::Parser::new(&text)?;
    let pattern = parser.pattern()?;
    let json = serde_json::to_string(&pattern).map_err(|e| e.to_string())?;
    print!("{}", json);
    Ok(())
}
