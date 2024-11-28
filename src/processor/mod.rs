use std::path::PathBuf;
use anyhow::Result;
use rayon::prelude::*;
use crate::linter::{Diagnostic, lint_file};
use crate::rules::Rule;

pub fn process_files(files: Vec<PathBuf>, rules: &[Box<dyn Rule + Sync>]) -> Result<Vec<Diagnostic>> {
    let diagnostics: Result<Vec<_>> = files.par_iter()
        .map(|file| -> Result<Vec<Diagnostic>> {
            lint_file(file, rules)
        })
        .collect();

    // Flatten the results into a single vector of diagnostics
    let mut all_diagnostics = Vec::new();
    for file_diagnostics in diagnostics? {
        all_diagnostics.extend(file_diagnostics);
    }
    
    Ok(all_diagnostics)
}

pub fn find_python_files(dir: &PathBuf) -> Result<Vec<PathBuf>> {
    use walkdir::WalkDir;

    let python_files: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file() && 
            e.path().extension().map_or(false, |ext| ext == "py")
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    Ok(python_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;
    use crate::rules::get_default_rules;

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
        
        // Create multiple test files with various issues
        let files = vec![
            ("test1.py", "import os\nimport sys\nx = 1  # only os is unused"),
            ("test2.py", "def badFunction(): pass  # bad naming"),
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
        
        // We should have at least one diagnostic per file
        assert!(diagnostics.len() >= 3, "Expected at least 3 diagnostics, got {}", diagnostics.len());
        
        // Verify we have different types of diagnostics
        let messages: Vec<_> = diagnostics.iter().map(|d| &d.message).collect();
        println!("Found diagnostics: {:?}", messages);
        
        // Check for unused import diagnostic
        assert!(messages.iter().any(|m| m.contains("Unused import")), 
            "No unused import diagnostic found in: {:?}", messages);
        
        // Check for naming convention diagnostic
        assert!(messages.iter().any(|m| m.contains("snake_case")), 
            "No naming convention diagnostic found in: {:?}", messages);
        
        // Check for line length diagnostic
        assert!(messages.iter().any(|m| m.contains("too long")), 
            "No line length diagnostic found in: {:?}", messages);

        Ok(())
    }

    #[test]
    fn test_parallel_processing_large_files() -> Result<()> {
        let dir = tempdir()?;
        let rules: Vec<Box<dyn Rule + Sync>> = get_default_rules()
            .into_iter()
            .map(|r| r as Box<dyn Rule + Sync>)
            .collect();
        
        // Create 10 large files with various issues
        let mut file_paths = Vec::new();
        for i in 0..10 {
            let file_path = dir.path().join(format!("large_file_{}.py", i));
            let mut file = File::create(&file_path)?;
            
            // Add an unused import at the start
            writeln!(file, "import os")?;
            writeln!(file, "import sys")?;
            writeln!(file, "from typing import List")?;
            
            // Write 1000 lines to each file
            for j in 0..1000 {
                writeln!(file, "sys.x_{}_{}_ = {}", i, j, j)?;
            }
            
            file_paths.push(file_path);
        }

        let diagnostics = process_files(file_paths, &rules)?;
        println!("Found diagnostics: {:?}", diagnostics);
        
        // Verify we found unused imports
        let messages: Vec<_> = diagnostics.iter().map(|d| &d.message).collect();
        assert!(messages.iter().any(|m| m.contains("Unused import")), 
            "No unused import diagnostic found in: {:?}", messages);
        assert!(diagnostics.len() > 0, "Expected at least one diagnostic");

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
}
