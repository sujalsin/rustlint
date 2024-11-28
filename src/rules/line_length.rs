use anyhow::Result;
use rustpython_parser::ast;
use crate::linter::Diagnostic;
use crate::linter::DiagnosticLevel;

pub struct LineLength {
    max_length: usize,
}

impl LineLength {
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }
}

impl super::Rule for LineLength {
    fn check(&self, _ast: &ast::Suite, source: &str) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = source.lines().collect();

        for (i, &line) in lines.iter().enumerate() {
            let line_num = i + 1;
            let line_length = line.chars().count();

            // Skip empty lines and lines that are just quotes from multiline strings
            if line.trim().is_empty() || line.trim() == "\"\"\"" {
                continue;
            }

            if line_length > self.max_length {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Warning,
                    message: format!(
                        "Line too long ({} > {} characters)",
                        line_length,
                        self.max_length
                    ),
                    line: line_num,
                    column: self.max_length + 1,
                    path: String::new(),
                });
            }
        }

        Ok(diagnostics)
    }
}
