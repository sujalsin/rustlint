use anyhow::Result;
use rustpython_parser::{ast, parser};
use std::{collections::HashSet, path::Path};

pub struct Linter {
    config: crate::config::Config,
}

impl Linter {
    pub fn new(config: crate::config::Config) -> Self {
        Self { config }
    }

    pub fn lint_file(&self, path: &Path) -> Result<Vec<Diagnostic>> {
        println!("Linting file: {:?}", path);
        let content = std::fs::read_to_string(path)?;
        println!("File content length: {}", content.len());
        let mut diagnostics = self.lint_source(&content, path)?;
        println!("Found diagnostics: {:?}", diagnostics);
        
        // Add file path to all diagnostics
        for diagnostic in &mut diagnostics {
            diagnostic.path = path.to_string_lossy().to_string();
        }
        
        Ok(diagnostics)
    }

    pub fn lint_source(&self, source: &str, path: &Path) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        // Always run style checks since they don't depend on valid syntax
        self.check_style(source, &mut diagnostics)?;

        // Try parsing the file for syntax and unused code checks
        match parser::parse_program(source, path.to_str().unwrap_or("unknown")) {
            Ok(ast) => {
                self.check_syntax(&ast, &mut diagnostics)?;
                self.check_unused(&ast, &mut diagnostics)?;
            }
            Err(e) => {
                // Handle syntax errors
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Error,
                    message: format!("Syntax error: {}", e),
                    line: 1, // TODO: Extract line number from error
                    column: 1,
                    path: String::new(),
                });
            }
        }

        Ok(diagnostics)
    }

    fn check_syntax(&self, _ast: &ast::Suite, _diagnostics: &mut Vec<Diagnostic>) -> Result<()> {
        // Basic syntax validation is handled by rustpython-parser
        // We'll add more advanced checks later
        Ok(())
    }

    fn check_style(&self, source: &str, diagnostics: &mut Vec<Diagnostic>) -> Result<()> {
        let lines: Vec<&str> = source.lines().collect();
        let max_length = self.config.rules.max_line_length;

        for (i, &line) in lines.iter().enumerate() {
            let line_num = i + 1;
            
            // Check line length
            let line_length = line.chars().count(); // Use char count instead of byte length
            println!("Line {}: length = {}, max = {}", line_num, line_length, max_length); // Debug print
            if line_length > max_length {
                println!("Found long line: {}", line); // Debug print
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Warning,
                    message: format!(
                        "Line too long ({} > {})",
                        line_length,
                        max_length
                    ),
                    line: line_num,
                    column: max_length + 1,
                    path: String::new(),
                });
            }

            // Check indentation
            let indent_size = line.chars()
                .take_while(|&c| c == ' ')
                .count();
            
            if indent_size > 0 && indent_size % 4 != 0 && !line.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Warning,
                    message: format!("Indentation of {} spaces should be a multiple of 4", indent_size),
                    line: line_num,
                    column: 1,
                    path: String::new(),
                });
            }

            // Check for mixed tabs and spaces
            if line.contains('\t') {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Warning,
                    message: "Line contains tabs (use spaces instead)".to_string(),
                    line: line_num,
                    column: line.find('\t').unwrap_or(0) + 1,
                    path: String::new(),
                });
            }

            // Check trailing whitespace
            if line.trim_end().len() < line.len() {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Warning,
                    message: "Trailing whitespace".to_string(),
                    line: line_num,
                    column: line.trim_end().len() + 1,
                    path: String::new(),
                });
            }
        }

        Ok(())
    }

    fn check_unused(&self, ast: &ast::Suite, diagnostics: &mut Vec<Diagnostic>) -> Result<()> {
        let mut defined_imports = HashSet::new();
        let mut used_names = HashSet::new();

        // First pass: collect all imports and defined names
        for stmt in ast.iter() {
            match &stmt.node {
                ast::StmtKind::Import { names } => {
                    for alias in names {
                        defined_imports.insert((
                            alias.node.name.to_string(),
                            stmt.location.row(),
                            stmt.location.column(),
                        ));
                    }
                }
                ast::StmtKind::ImportFrom { names, .. } => {
                    for alias in names {
                        defined_imports.insert((
                            alias.node.name.to_string(),
                            stmt.location.row(),
                            stmt.location.column(),
                        ));
                    }
                }
                _ => {}
            }
        }

        // Second pass: collect all used names
        self.collect_used_names_from_suite(ast, &mut used_names);

        // Report unused imports
        for (import, line, col) in defined_imports {
            if !used_names.contains(&import) {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Warning,
                    message: format!("Unused import '{}'", import),
                    line,
                    column: col,
                    path: String::new(),
                });
            }
        }

        Ok(())
    }

    fn collect_used_names_from_suite(&self, suite: &ast::Suite, used_names: &mut HashSet<String>) {
        for stmt in suite {
            match &stmt.node {
                ast::StmtKind::Expr { value } => {
                    self.collect_used_names(value, used_names);
                }
                ast::StmtKind::FunctionDef { body, .. } => {
                    for stmt in body {
                        if let ast::StmtKind::Expr { value } = &stmt.node {
                            self.collect_used_names(value, used_names);
                        }
                    }
                }
                ast::StmtKind::Assign { targets, value, .. } => {
                    for target in targets {
                        self.collect_used_names(target, used_names);
                    }
                    self.collect_used_names(value, used_names);
                }
                _ => {}
            }
        }
    }

    fn collect_used_names(&self, node: &ast::Expr, used_names: &mut HashSet<String>) {
        match &node.node {
            ast::ExprKind::Name { id, .. } => {
                used_names.insert(id.to_string());
            }
            ast::ExprKind::Call { func, args, .. } => {
                self.collect_used_names(func, used_names);
                for arg in args {
                    self.collect_used_names(arg, used_names);
                }
            }
            ast::ExprKind::Attribute { value, attr, .. } => {
                used_names.insert(attr.to_string());
                self.collect_used_names(value, used_names);
            }
            _ => {}
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
}

#[derive(Debug)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub path: String,
}

pub fn lint_file(path: &Path, rules: &[Box<dyn crate::rules::Rule + Sync>]) -> Result<Vec<Diagnostic>> {
    let content = std::fs::read_to_string(path)?;
    let mut diagnostics = Vec::new();

    // Try parsing the file
    match parser::parse_program(&content, path.to_str().unwrap_or("unknown")) {
        Ok(ast) => {
            // Apply each rule
            for rule in rules {
                let mut rule_diagnostics = rule.check(&ast, &content)?;
                diagnostics.append(&mut rule_diagnostics);
            }
        }
        Err(e) => {
            // Handle syntax errors
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Error,
                message: format!("Syntax error: {}", e),
                line: 1,
                column: 1,
                path: String::new(),
            });
        }
    }

    // Add file path to all diagnostics
    for diagnostic in &mut diagnostics {
        diagnostic.path = path.to_string_lossy().to_string();
    }

    Ok(diagnostics)
}
