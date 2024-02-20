use std::{ path::PathBuf, time::Duration };
use anyhow::Result;
use ignore::gitignore::Gitignore;

// parse patterns from gitignore
// iterate files & folders
// if folder has gitignore, then call recursive and pass it
// if folder doesn't have gitignore, continue search for gitignores in sub folders

pub struct Cleaner {
    path: PathBuf,
    delete: bool,
    quiet: bool,
    ignore_errors: bool,
}

impl Cleaner {
    pub fn new(path: PathBuf, delete: bool, quiet: bool, ignore_errors: bool) -> Self {
        Cleaner {
            path,
            delete,
            quiet,
            ignore_errors,
        }
    }

    fn clean_with_gitignore(&self, path: PathBuf, gitignore: Option<Gitignore>) -> Result<()> {
        let dir = std::fs::read_dir(path.clone());
        if let Err(err) = dir {
            if self.ignore_errors {
                eprintln!("❌ Error processing directory {}: {}", path.display(), err);
                return Ok(());
            } else {
                panic!("{}", err);
            }
        }
        for entry in dir
            .unwrap()
            .into_iter()
            .filter_map(|e| e.ok()) {
            // if we have gitignore, try to clean
            log::trace!("entry {}", entry.path().display());
            if let Some(ref gitignore) = gitignore {
                if gitignore.matched(entry.path(), entry.path().is_dir()).is_ignore() {
                    if !self.quiet {
                        if entry.path().is_dir() {
                            println!("🗂️  {}", entry.path().display());
                        } else {
                            println!("📄 {}", entry.path().display());
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
                log::debug!(
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
                    log::debug!("Visiting {} with parent gitignore.", entry.path().display());
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

    pub fn clean(&self) -> Result<()> {
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
            return Ok(());
        }
        self.clean_with_gitignore(self.path.clone(), None)?;

        Ok(())
    }
}
