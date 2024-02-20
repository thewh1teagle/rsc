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
    let cleaner = clean::Cleaner::new(args.path, args.delete, args.quiet);
    cleaner.clean()?;

    Ok(())
}
