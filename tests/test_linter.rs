use rustlint::{config::Config, linter::{Diagnostic, DiagnosticLevel}};
use anyhow::Result;
use std::path::PathBuf;

#[test]
fn test_style_issues() -> anyhow::Result<()> {
    let config = Config::default();
    let linter = rustlint::linter::Linter::new(config);
    let path = PathBuf::from("tests/test_files/style_issues.py");
    
    let diagnostics = linter.lint_file(&path)?;
    
    // We should have multiple style issues
    assert!(!diagnostics.is_empty(), "Should detect style issues");
    
    let warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.level, DiagnosticLevel::Warning))
        .collect();
    
    println!("Found warnings: {:?}", warnings);
    
    // Check for line length issues
    let has_line_length_warning = warnings.iter().any(|d| d.message.contains("Line too long"));
    println!("Has line length warning: {}", has_line_length_warning);
    assert!(has_line_length_warning, "Should detect line length issues");
    
    // Check for indentation issues
    assert!(
        warnings.iter().any(|d| d.message.contains("Indentation")),
        "Should detect indentation issues"
    );
    
    Ok(())
}

#[test]
fn test_unused_code() -> anyhow::Result<()> {
    let config = Config::default();
    let linter = rustlint::linter::Linter::new(config);
    let path = PathBuf::from("tests/test_files/unused_code.py");
    
    let diagnostics = linter.lint_file(&path)?;
    
    // We should have multiple unused code issues
    assert!(!diagnostics.is_empty(), "Should detect unused code");
    
    let warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.level, DiagnosticLevel::Warning))
        .collect();
    
    println!("Found warnings: {:?}", warnings);
    
    // Check for unused imports
    let has_unused_import_sys = warnings.iter().any(|d| d.message.contains("Unused import 'sys'"));
    println!("Has unused import 'sys': {}", has_unused_import_sys);
    assert!(has_unused_import_sys, "Should detect unused import 'sys'");
    
    let has_unused_import_json = warnings.iter().any(|d| d.message.contains("Unused import 'json'"));
    println!("Has unused import 'json': {}", has_unused_import_json);
    assert!(has_unused_import_json, "Should detect unused import 'json'");
    
    Ok(())
}

#[test]
fn test_syntax_errors() -> anyhow::Result<()> {
    let config = Config::default();
    let linter = rustlint::linter::Linter::new(config);
    let path = PathBuf::from("tests/test_files/syntax_errors.py");
    
    let diagnostics = linter.lint_file(&path)?;
    
    // We should have syntax errors
    assert!(!diagnostics.is_empty(), "Should detect syntax errors");
    
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.level, DiagnosticLevel::Error))
        .collect();
    
    println!("Found errors: {:?}", errors);
    
    // Check for syntax errors
    assert!(
        !errors.is_empty(),
        "Should detect syntax errors as errors, not warnings"
    );
    
    let has_syntax_error = errors.iter().any(|d| d.message.contains("Syntax error"));
    println!("Has syntax error: {}", has_syntax_error);
    assert!(has_syntax_error, "Should detect basic syntax errors");
    
    Ok(())
}

#[test]
fn test_diagnostic_level() {
    assert_ne!(DiagnosticLevel::Error, DiagnosticLevel::Warning);
    assert_eq!(DiagnosticLevel::Error, DiagnosticLevel::Error);
    assert_eq!(DiagnosticLevel::Warning, DiagnosticLevel::Warning);
}
