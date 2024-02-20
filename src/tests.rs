use std::fs::{ self, File };
use std::io::{ self, Write };
use std::path::PathBuf;
use crate::clean;
use anyhow::Result;

#[test]
fn delete_ignored_files() -> Result<()> {
    // Create a temporary directory
    let temp_dir = tempfile::tempdir()?;
    let temp_dir_path = temp_dir.path();

    // Create a .gitignore file with 'ignored.txt'
    let mut gitignore_file = File::create(temp_dir_path.join(".gitignore"))?;
    gitignore_file.write_all(b"ignored.txt\n")?;

    // Create 'ignored.txt' and 'not_ignored.txt' inside the directory
    File::create(temp_dir_path.join("ignored.txt"))?;
    File::create(temp_dir_path.join("not_ignored.txt"))?;

    // Run the cleaner on that folder
    let cleaner = clean::Cleaner::new(temp_dir_path.to_path_buf(), true, true);
    cleaner.clean()?;

    // Assert that 'ignored.txt' is not there but 'not_ignored.txt' is there
    assert!(!temp_dir_path.join("ignored.txt").exists());
    assert!(temp_dir_path.join("not_ignored.txt").exists());

    Ok(())
}

#[test]
fn delete_inner_ignored_file() -> Result<()> {
    // Create a temporary directory
    let temp_dir = tempfile::tempdir()?;
    let temp_dir_path = temp_dir.path();

    // Ensure that the directory 'some/sub/folder' exists
    let inner_folder_path = temp_dir_path.join("some/sub/folder");
    std::fs::create_dir_all(&inner_folder_path)?;

    // Create a .gitignore file with 'inner_ignored.txt'
    let mut gitignore_file = File::create(inner_folder_path.join(".gitignore"))?;
    gitignore_file.write_all(b"inner_ignored.txt\n")?;

    // Create 'inner_ignored.txt' inside the directory
    File::create(inner_folder_path.join("inner_ignored.txt"))?;
    File::create(inner_folder_path.join("not_inner_ignored.txt"))?;

    // Run the cleaner on that folder
    let cleaner = clean::Cleaner::new(temp_dir_path.to_path_buf(), true, true);
    cleaner.clean()?;

    // Assert that 'inner_ignored.txt' is not there but 'not_inner_ignored.txt' is there
    assert!(!inner_folder_path.join("inner_ignored.txt").exists());
    assert!(inner_folder_path.join("not_inner_ignored.txt").exists());

    Ok(())
}

#[test]
fn delete_ignored_files_and_folder() -> Result<()> {
    // Create a temporary directory
    let temp_dir = tempfile::tempdir()?;
    let temp_dir_path = temp_dir.path();

    // Create a .gitignore file with 'ignored.txt'
    let mut gitignore_file = File::create(temp_dir_path.join(".gitignore"))?;
    gitignore_file.write_all(b"ignored.txt\nfolder\n")?;

    // Create 'ignored.txt', 'not_ignored.txt', and 'folder' inside the directory
    File::create(temp_dir_path.join("ignored.txt"))?;
    File::create(temp_dir_path.join("not_ignored.txt"))?;
    std::fs::create_dir(temp_dir_path.join("folder"))?;

    // Run the cleaner on that folder
    let cleaner = clean::Cleaner::new(temp_dir_path.to_path_buf(), true, true);
    cleaner.clean()?;

    // Assert that 'ignored.txt' is not there, 'not_ignored.txt' is there, and 'folder' is not there
    assert!(temp_dir_path.join("not_ignored.txt").exists());
    assert!(!temp_dir_path.join("ignored.txt").exists());
    assert!(!temp_dir_path.join("folder").exists());

    Ok(())
}
