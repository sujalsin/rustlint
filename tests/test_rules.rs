use anyhow::Result;
use rustlint::rules::{UnusedImports, LineLength, NamingConventions, Rule};
use rustlint::linter::DiagnosticLevel;
use std::path::PathBuf;
use std::fs;

fn load_test_file(name: &str) -> Result<(String, PathBuf)> {
    let path = PathBuf::from(format!("tests/test_files/{}", name));
    let content = fs::read_to_string(&path)?;
    Ok((content, path))
}

#[test]
fn test_unused_imports() -> Result<()> {
    let code = r#"
import os
import sys
from typing import List, Dict
from pathlib import Path as PathLib
import json as js

def main():
    sys_path = sys.path
    my_list: List = []
    return my_list
"#;
    let ast = rustpython_parser::parser::parse_program(code, "<string>")?;
    let rule = UnusedImports;
    let diagnostics = rule.check(&ast, code)?;

    assert_eq!(diagnostics.len(), 4);
    assert!(diagnostics.iter().any(|d| d.message.contains("os")));
    assert!(diagnostics.iter().any(|d| d.message.contains("Dict")));
    assert!(diagnostics.iter().any(|d| d.message.contains("PathLib")));
    assert!(diagnostics.iter().any(|d| d.message.contains("js")));
    Ok(())
}

#[test]
fn test_line_length() -> Result<()> {
    let code = r#"
# This is a very long comment line that should definitely trigger the line length warning because it's way too long
def short_line():
    pass
"""This is a multiline string that contains a very long line that should trigger the line length warning because it's quite lengthy
Normal line
Another normal line
Yet another normal line
And finally we have a very long line that should definitely trigger the line length warning because it goes beyond the limit
"""
"#;
    let ast = rustpython_parser::parser::parse_program(code, "<string>")?;
    let rule = LineLength::new(88);
    let diagnostics = rule.check(&ast, code)?;

    assert_eq!(diagnostics.len(), 3);
    assert_eq!(diagnostics[0].line, 2); // Comment line
    assert_eq!(diagnostics[1].line, 5); // Multiline string line
    assert_eq!(diagnostics[2].line, 9); // Long line in multiline string
    Ok(())
}

#[test]
fn test_naming_conventions() -> Result<()> {
    let code = r#"
def badFunction():
    pass

class badclass:
    pass

class GoodClass:
    def good_function():
        pass

def good_name():
    BAD_var = 1
    good_var = 2
    CONSTANT = 3
"#;
    let ast = rustpython_parser::parser::parse_program(code, "<string>")?;
    let rule = NamingConventions;
    let diagnostics = rule.check(&ast, code)?;

    assert_eq!(diagnostics.len(), 3);
    assert!(diagnostics.iter().any(|d| d.message.contains("badFunction")));
    assert!(diagnostics.iter().any(|d| d.message.contains("badclass")));
    assert!(diagnostics.iter().any(|d| d.message.contains("BAD_var")));
    Ok(())
}


#[test]
fn test_multiple_rules() -> Result<()> {
    let rules: Vec<Box<dyn Rule>> = vec![
        Box::new(UnusedImports),
        Box::new(LineLength::new(88)),
        Box::new(NamingConventions),
    ];

    // Test each file with all rules
    let test_files = ["unused_imports.py", "line_length.py", "naming_conventions.py"];
    
    for file in test_files {
        let (content, path) = load_test_file(file)?;
        let ast = rustpython_parser::parser::parse_program(&content, &path.to_string_lossy())?;

        let mut all_diagnostics = Vec::new();
        for rule in &rules {
            let diagnostics = rule.check(&ast, &content)?;
            all_diagnostics.extend(diagnostics);
        }

        // Verify we get at least some diagnostics for each file
        assert!(!all_diagnostics.is_empty(), "No diagnostics found for {}", file);
        
        // Verify all diagnostics have valid line numbers
        assert!(all_diagnostics.iter().all(|d| d.line > 0));
    }

    Ok(())
}
