use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;
use valis_core::modules::projects::venv::{rebuild, VirtualEnv};

#[test]
fn test_rebuild() {
    // Create a temporary directory
    let dir = tempdir().unwrap();
    let venv_dir = dir.path().join(".virtualenvs");
    let project_dir = venv_dir.join("project");
    let requirements_path = project_dir.join("requirements.txt");

    // Mock virtualenv and requirements.txt
    std::fs::create_dir_all(&project_dir).unwrap();
    let mut file = File::create(&requirements_path).unwrap();
    writeln!(file, "requests").unwrap();

    // Create a mock VirtualEnv
    let venv = VirtualEnv {
        name: String::from("project"),
        location: project_dir.clone(),
        root: dir.path().to_path_buf(),
        requirements: requirements_path.clone(),
    };

    // Run the rebuild function
    rebuild(venv);

    // Check that the virtualenv has been deleted and recreated
    assert!(!project_dir.exists());
    assert!(requirements_path.exists());

    // Delete temporary directory
    dir.close().unwrap();
}
