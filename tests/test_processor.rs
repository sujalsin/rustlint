use anyhow::Result;
use rustlint::processor::{process_files, find_python_files};
use rustlint::rules::{get_default_rules, Rule};
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_find_python_files() -> Result<()> {
    let dir = tempdir()?;
    
    // Create test files with different extensions
    let files = vec![
        ("test1.py", "x = 1"),
        ("test2.py", "y = 2"),
        ("not_python.txt", "hello"),
        ("nested/test3.py", "z = 3"),
    ];

    for (path, content) in files {
        let full_path = dir.path().join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(full_path)?;
        file.write_all(content.as_bytes())?;
    }

    let python_files = find_python_files(&dir.path().to_path_buf())?;
    
    // Should find exactly 3 Python files
    assert_eq!(python_files.len(), 3);
    
    // All files should end with .py
    assert!(python_files.iter().all(|p| p.extension().unwrap() == "py"));
    
    // Should include the nested file
    assert!(python_files.iter().any(|p| p.to_string_lossy().contains("nested")));

    Ok(())
}

#[test]
fn test_parallel_processing() -> Result<()> {
    let dir = tempdir()?;
    let rules: Vec<Box<dyn Rule + Sync>> = get_default_rules()
        .into_iter()
        .map(|r| r as Box<dyn Rule + Sync>)
        .collect();
    
    // Create test files with various issues
    let files = vec![
        ("test1.py", "import os\nimport sys\n\nprint('hello')  # os and sys are unused"),
        ("test2.py", "def badFunction():\n    x = 1  # bad naming convention"),
        ("test3.py", "x = 'this is a very long line that should definitely trigger our line length warning because it is too long'"),
    ];

    let mut file_paths = Vec::new();
    for (path, content) in files {
        let file_path = dir.path().join(path);
        let mut file = File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
        file_paths.push(file_path);
    }

    let diagnostics = process_files(file_paths, &rules)?;
    
    // Debug print messages
    println!("Found diagnostics:");
    for d in &diagnostics {
        println!("  - {} (in {})", d.message, d.path);
    }
    
    // We should have at least one diagnostic per file
    assert!(diagnostics.len() >= 3, "Expected at least 3 diagnostics, got {}", diagnostics.len());
    
    // Verify we have different types of diagnostics
    let messages: Vec<_> = diagnostics.iter().map(|d| &d.message).collect();
    
    // Check for unused import diagnostic
    assert!(messages.iter().any(|m| m.contains("Unused import")), 
           "No unused import diagnostic found in messages: {:?}", messages);
    
    // Check for naming convention diagnostic
    assert!(messages.iter().any(|m| m.contains("snake_case")), 
           "No naming convention diagnostic found in messages: {:?}", messages);
    
    // Check for line length diagnostic
    assert!(messages.iter().any(|m| m.contains("too long")), 
           "No line length diagnostic found in messages: {:?}", messages);
    
    Ok(())
}

#[test]
fn test_parallel_processing_large_files() -> Result<()> {
    let dir = tempdir()?;
    let rules: Vec<Box<dyn Rule + Sync>> = get_default_rules()
        .into_iter()
        .map(|r| r as Box<dyn Rule + Sync>)
        .collect();
    
    // Create 10 large files with deliberate issues
    let mut file_paths = Vec::new();
    for i in 0..10 {
        let file_path = dir.path().join(format!("large_file_{}.py", i));
        let mut file = File::create(&file_path)?;
        
        // Add unused imports
        writeln!(file, "import os  # unused import")?;
        writeln!(file, "import sys  # unused import")?;
        writeln!(file, "import json  # unused import")?;
        
        // Add some code with naming convention issues
        for j in 0..100 {
            writeln!(file, "def badFunction_{}():", j)?;
            writeln!(file, "    badVariable = {}", j)?;
            writeln!(file, "    print(badVariable)")?;
        }
        
        file_paths.push(file_path);
    }

    let diagnostics = process_files(file_paths, &rules)?;
    
    // Debug print messages
    println!("Found diagnostics in large files:");
    for d in &diagnostics {
        println!("  - {} (in {})", d.message, d.path);
    }
    
    // Verify we found unused imports
    let messages: Vec<_> = diagnostics.iter().map(|d| &d.message).collect();
    assert!(messages.iter().any(|m| m.contains("Unused import")), 
           "No unused import diagnostic found in messages: {:?}", messages);
    
    // Verify processing completed and found issues
    assert!(!diagnostics.is_empty(), 
            "Expected diagnostics for large files, but found none");
    assert!(diagnostics.len() > 10, 
            "Expected at least one diagnostic per file, got {}", diagnostics.len());

    Ok(())
}

#[test]
fn test_empty_directory() -> Result<()> {
    let dir = tempdir()?;
    let python_files = find_python_files(&dir.path().to_path_buf())?;
    assert!(python_files.is_empty());
    Ok(())
}

#[test]
fn test_process_empty_file_list() -> Result<()> {
    let rules: Vec<Box<dyn Rule + Sync>> = get_default_rules()
        .into_iter()
        .map(|r| r as Box<dyn Rule + Sync>)
        .collect();
    let diagnostics = process_files(Vec::new(), &rules)?;
    assert!(diagnostics.is_empty());
    Ok(())
}
