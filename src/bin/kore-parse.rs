use std::error::Error;
use std::fs::read_to_string;
use std::path::PathBuf;

use clap::Parser;

use kframework::kore;

#[derive(Parser)]
/// Parse a .kore file
struct Args {
    /// Path of .kore file to parse
    path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let text = read_to_string(args.path)?;
    let mut parser = kore::Parser::new(&text)?;
    let _definition = parser.definition()?;
    Ok(())
}
