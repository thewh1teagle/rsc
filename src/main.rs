use std::path::PathBuf;

use log;
use clap::{ command, Parser };
use env_logger;
use anyhow::Result;
mod clean;

#[cfg(test)]
mod tests;

/// Rusty File Cleaner is a cross-platform Rust tool for cleaning operating system files.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Root Path to clean from
    path: PathBuf,

    /// Enable deletion
    #[arg(short, long)]
    delete: bool,

    /// Quiet mode
    #[arg(short, long, action)]
    quiet: bool,

    /// Ignore errors such as permission denied
    #[arg(short, long, action)]
    ignore_errors: bool,

    /// Skip nested gitignores scan
    #[arg(long, action)]
    skip_nested: bool,

    /// Regex patterns to skip deletion
    #[arg(long, action)]
    skip_patterns: Option<Vec<String>>,

    /// Calculate delete size and display per files and total
    #[arg(long, action)]
    calculate_size: bool,
}

fn main() -> Result<()> {
    env_logger::init();
    log::trace!("rsc started");
    let args = Args::parse();

    // Ensure path exists
    if !args.path.exists() {
        eprintln!("❌ path {} does not exists!", args.path.display());
        std::process::exit(1);
    }

    // Ensure path is directory
    if !args.path.is_dir() {
        eprintln!("❌ path {} is not a directory!", args.path.display());
        std::process::exit(1);
    }
    let mut cleaner = clean::Cleaner::try_create(
        args.path,
        args.delete,
        args.quiet,
        args.ignore_errors,
        args.skip_nested,
        args.skip_patterns,
        args.calculate_size
    )?;
    cleaner.clean()?;

    Ok(())
}
