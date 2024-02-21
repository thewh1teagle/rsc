use std::fs::File;
use std::io::Write;
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
    let mut cleaner = clean::Cleaner::try_create(
        temp_dir_path.to_path_buf(),
        true,
        true,
        false,
        false,
        None,
        false
    )?;
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
    let mut cleaner = clean::Cleaner::try_create(
        temp_dir_path.to_path_buf(),
        true,
        true,
        false,
        false,
        None,
        false
    )?;
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
    let mut cleaner = clean::Cleaner::try_create(
        temp_dir_path.to_path_buf(),
        true,
        true,
        false,
        false,
        None,
        false
    )?;
    cleaner.clean()?;

    // Assert that 'ignored.txt' is not there, 'not_ignored.txt' is there, and 'folder' is not there
    assert!(temp_dir_path.join("not_ignored.txt").exists());
    assert!(!temp_dir_path.join("ignored.txt").exists());
    assert!(!temp_dir_path.join("folder").exists());

    Ok(())
}

#[test]
fn parent_gitignore_not_touching_sub_gitignore() -> Result<()> {
    // Create a temporary directory
    let temp_dir = tempfile::tempdir()?;
    let temp_dir_path = temp_dir.path();
    let inner_path = temp_dir_path.join("inner");

    // Create a .gitignore file with content 'ignored.txt'
    let mut gitignore_file = File::create(temp_dir_path.join(".gitignore"))?;
    gitignore_file.write_all(b"inner.txt\n")?;

    // Create inner folder
    std::fs::create_dir(inner_path.clone())?;
    // empty gitignore
    let mut gitignore_file = File::create(inner_path.clone().join(".gitignore"))?;
    gitignore_file.write_all(b"nothing\n")?;
    // Create inner.txt file in inner folder
    File::create(inner_path.clone().join("inner.txt"))?;

    // Run the cleaner on parent folder
    let mut cleaner = clean::Cleaner::try_create(
        temp_dir_path.to_path_buf(),
        true,
        true,
        false,
        false,
        None,
        false
    )?;
    cleaner.clean()?;

    // Assert that 'inner.txt' is there, and parent gitignore doesn't affect it since we have inner gitignore
    assert!(inner_path.join("inner.txt").exists());

    Ok(())
}

#[test]
fn no_files_deleted_when_no_gitignore() -> Result<()> {
    // Create a temporary directory
    let temp_dir = tempfile::tempdir()?;
    let temp_dir_path = temp_dir.path();

    // Create some test files
    let files_to_create = ["file1.txt", "file2.txt", "file3.txt"];
    for file in files_to_create.iter() {
        File::create(temp_dir_path.join(file))?;
    }

    // Run the cleaner on the parent folder without using gitignore options
    let mut cleaner = clean::Cleaner::try_create(
        temp_dir_path.to_path_buf(),
        false,
        false,
        false,
        false,
        None,
        false
    )?;
    cleaner.clean()?;

    // Check if all the created files still exist
    for file in files_to_create.iter() {
        assert!(temp_dir_path.join(file).exists());
    }

    Ok(())
}

#[test]
fn ensure_symlink_files_never_deleted() -> Result<()> {
    // Create a temporary directory
    let temp_dir = tempfile::tempdir()?;
    let temp_dir_path = temp_dir.path();

    // Create a symbolic link to a file
    let symlink_target_path = temp_dir_path.join("target_file.txt");
    File::create(&symlink_target_path)?;

    // Create gitignore
    let mut gitignore_file = File::create(temp_dir_path.join(".gitignore"))?;
    gitignore_file.write_all(b"symlink.txt\n")?;

    let symlink_path = temp_dir_path.join("symlink.txt");
    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(&symlink_target_path, &symlink_path)?;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_file;
        symlink_file(&symlink_target_path, &symlink_path)?;
    }

    // Run the cleaner on that folder
    let mut cleaner = clean::Cleaner::try_create(
        temp_dir_path.to_path_buf(),
        true,
        true,
        false,
        false,
        None,
        false
    )?;
    cleaner.clean()?;

    // Assert that the symbolic link still exists
    assert!(symlink_path.exists());

    Ok(())
}

#[test]
fn ensure_symlink_folders_never_deleted() -> Result<()> {
    // Create a temporary directory
    let temp_dir = tempfile::tempdir()?;
    let temp_dir_path = temp_dir.path();

    // Create a new temporary directory to be symlinked
    let target_folder = tempfile::tempdir()?;
    let target_folder_path = target_folder.path();

    // Create gitignore
    let mut gitignore_file = File::create(temp_dir_path.join(".gitignore"))?;
    gitignore_file.write_all(b"symlinked_folder/\n")?;

    let symlink_path = temp_dir_path.join("symlinked_folder");
    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(&target_folder_path, &symlink_path)?;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_dir;
        symlink_dir(&target_folder_path, &symlink_path)?;
    }

    // Run the cleaner on that folder
    let mut cleaner = clean::Cleaner::try_create(
        temp_dir_path.to_path_buf(),
        true,
        true,
        false,
        false,
        None,
        false
    )?;
    cleaner.clean()?;

    // Assert that the symbolic link still exists
    assert!(symlink_path.exists());

    Ok(())
}
