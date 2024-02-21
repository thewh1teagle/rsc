use std::{ path::{ Path, PathBuf }, time::Duration };
use anyhow::{ Context, Result };
use ignore::gitignore::Gitignore;
use regex::Regex;
use fs_extra;
use bytesize::{ self, ByteSize };

// parse patterns from gitignore
// iterate files & folders
// if folder has gitignore, then call recursive and pass it
// if folder doesn't have gitignore, continue search for gitignores in sub folders

pub struct Cleaner {
    path: PathBuf,
    delete: bool,
    quiet: bool,
    ignore_errors: bool,
    skip_nested: bool,
    skip_patterns: Vec<Regex>,
    calculate_size: bool,
    total_size: u64,
}

trait CalculateSize {
    fn size(&self) -> Result<u64>;
}

impl CalculateSize for Path {
    fn size(&self) -> Result<u64> {
        if self.is_dir() {
            let size = fs_extra::dir::get_size(self)?;
            return Ok(size);
        }
        Ok(self.metadata()?.len())
    }
}

fn human_size(size: u64) -> Result<String> {
    let human_size = ByteSize(size).to_string();
    Ok(human_size)
}

impl Cleaner {
    pub fn try_create(
        path: PathBuf,
        delete: bool,
        quiet: bool,
        ignore_errors: bool,
        skip_nested: bool,
        skip_patterns: Option<Vec<String>>,
        calculate_size: bool
    ) -> Result<Self> {
        let skip_patterns: Vec<Regex> = skip_patterns
            .unwrap_or_default()
            .iter()
            .map(|e| Regex::new(e).unwrap())
            .collect();
        // println!("{:?}", extra_ignore_patterns);
        let cleaner = Cleaner {
            path,
            delete,
            quiet,
            ignore_errors,
            skip_nested,
            skip_patterns,
            calculate_size,
            total_size: 0,
        };
        Ok(cleaner)
    }

    fn clean_with_gitignore(&mut self, path: PathBuf, gitignore: Option<Gitignore>) -> Result<()> {
        let dir = std::fs::read_dir(path.clone());
        if let Err(err) = dir {
            if self.ignore_errors {
                eprintln!("❌ Error processing directory {}: {}", path.display(), err);
                return Ok(());
            } else {
                panic!("{}", err);
            }
        }
        'outer: for entry in dir
            .unwrap()
            .into_iter()
            .filter_map(|e| e.ok()) {
            // if we have gitignore, try to clean
            log::trace!("entry {}", entry.path().display());
            if let Some(ref gitignore) = gitignore {
                let full = entry.path().canonicalize().context("cant get full path")?;
                let full = full.as_path().to_str().context("cant convet path to str")?;

                for pattern in self.skip_patterns.clone() {
                    if pattern.is_match(full) {
                        log::debug!(
                            "Skipping path {} match pattern {}",
                            entry.path().display(),
                            pattern
                        );
                        continue 'outer;
                    }
                }

                if gitignore.matched(entry.path(), entry.path().is_dir()).is_ignore() {
                    if !self.quiet {
                        let size_str = if self.calculate_size {
                            let size = entry.path().size()?;
                            self.total_size += size;
                            let human_size = human_size(size)?;
                            format!(" - {}", human_size)
                        } else {
                            "".to_string()
                        };
                        if entry.path().is_dir() {
                            println!("🗂️  {}{}", entry.path().display(), size_str);
                        } else {
                            println!("📄 {}{}", entry.path().display(), size_str);
                        }
                    }

                    // Danger place
                    if self.delete {
                        if entry.path().is_symlink() {
                            continue;
                        }
                        if entry.path().is_dir() {
                            std::fs::remove_dir_all(entry.path())?;
                        } else if entry.path().is_file() {
                            std::fs::remove_file(entry.path())?;
                        }
                    }
                    continue;
                }
            }
            // Try to find gitignore
            let sub_gitignore = entry.path().join(".gitignore");

            if sub_gitignore.clone().exists() {
                let (new_gitignore, error) = ignore::gitignore::Gitignore::new(
                    sub_gitignore.clone()
                );
                if error.is_some() {
                    eprintln!(
                        "❌ Failed to parse gitignore at {}. skipping dir...",
                        sub_gitignore.clone().display()
                    );
                    // don't get into it
                    continue;
                }
                // Get into directory with new gitignore
                if self.skip_nested {
                    log::debug!(
                        "Skipping {} with gitignore {}",
                        entry.path().display(),
                        sub_gitignore.display()
                    );
                    continue;
                }
                log::trace!(
                    "Visiting {} with new gitignore {}",
                    entry.path().display(),
                    sub_gitignore.display()
                );
                let result = self.clean_with_gitignore(
                    entry.path().to_path_buf(),
                    Some(new_gitignore)
                );
                if let Some(err) = result.err() {
                    if self.ignore_errors {
                        eprintln!("❌ Error processing file {}: {}", entry.path().display(), err);
                        continue;
                    } else {
                        panic!("{}", err);
                    }
                }
            } else {
                // Just get into the directory with current gitignore

                if entry.path().is_dir() {
                    if let Some(gitignore) = gitignore.clone() {
                        if gitignore.matched(entry.path(), entry.path().is_dir()).is_ignore() {
                            continue;
                        }
                    }
                    // Visit nested with new gitignore
                    log::trace!("Visiting {} with parent gitignore.", entry.path().display());
                    let result = self.clean_with_gitignore(
                        entry.path().to_path_buf(),
                        gitignore.clone()
                    );
                    if let Some(err) = result.err() {
                        if self.ignore_errors {
                            eprintln!(
                                "❌ Error processing file {}: {}",
                                entry.path().display(),
                                err
                            );
                            continue;
                        } else {
                            panic!("{}", err);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn clean(&mut self) -> Result<()> {
        if !self.delete {
            println!("🚫 Running in dry-run mode. Pass --delete to actually delete.");
            std::thread::sleep(Duration::from_millis(500));
        }
        let root_gitignore = self.path.join(".gitignore");
        if root_gitignore.exists() {
            let (new_gitignore, error) = ignore::gitignore::Gitignore::new(root_gitignore.clone());
            if error.is_some() {
                eprintln!(
                    "❌ Failed to parse gitignore at {}. skipping dir...",
                    root_gitignore.display()
                );
                // don't get into it
                return Ok(());
            }
            self.clean_with_gitignore(self.path.clone(), Some(new_gitignore))?;
            if self.calculate_size {
                let human_size = human_size(self.total_size)?;
                println!("💾 Total: {}", human_size);
            }
            return Ok(());
        }
        self.clean_with_gitignore(self.path.clone(), None)?;
        if self.calculate_size {
            let human_size = human_size(self.total_size)?;
            println!("💾 Total: {}", human_size);
        }
        Ok(())
    }
}
