use anyhow::Result;
use rustpython_parser::ast;
use crate::linter::Diagnostic;
use crate::linter::DiagnosticLevel;

pub struct NamingConventions;

impl NamingConventions {
    fn is_snake_case(name: &str) -> bool {
        let chars: Vec<char> = name.chars().collect();
        if chars.is_empty() || chars[0].is_numeric() {
            return false;
        }
        chars
            .iter()
            .all(|&c| c.is_lowercase() || c.is_numeric() || c == '_')
    }

    fn is_pascal_case(name: &str) -> bool {
        let mut chars = name.chars();
        if let Some(first_char) = chars.next() {
            if !first_char.is_uppercase() {
                return false;
            }
            chars.all(|c| c.is_alphanumeric())
        } else {
            false
        }
    }

    fn is_constant(name: &str) -> bool {
        let chars: Vec<char> = name.chars().collect();
        if chars.is_empty() {
            return false;
        }
        chars
            .iter()
            .all(|&c| c.is_uppercase() || c.is_numeric() || c == '_')
    }

    fn is_valid_variable_name(name: &str) -> bool {
        // A variable name should either be:
        // 1. A proper snake_case variable (all lowercase with underscores)
        // 2. A proper constant (all uppercase with underscores)
        // Any mix of upper and lower case that doesn't fit these patterns is wrong
        let has_uppercase = name.chars().any(|c| c.is_uppercase());
        let has_lowercase = name.chars().any(|c| c.is_lowercase());

        // Special case: If it has both upper and lower case letters,
        // it must be a proper snake_case or constant
        if has_uppercase && has_lowercase {
            return false;
        }

        // Otherwise, it must be either all snake_case or all constant
        Self::is_snake_case(name) || Self::is_constant(name)
    }

    fn check_statements(&self, stmts: &[ast::Stmt], diagnostics: &mut Vec<Diagnostic>) {
        for stmt in stmts {
            match &stmt.node {
                ast::StmtKind::FunctionDef { name, body, .. } => {
                    if !Self::is_snake_case(name) {
                        println!("Found bad function name: {}", name);
                        diagnostics.push(Diagnostic {
                            level: DiagnosticLevel::Warning,
                            message: format!("Function '{}' should use snake_case", name),
                            line: stmt.location.row(),
                            column: 1,
                            path: String::new(),
                        });
                    }
                    // Recurse into function body
                    self.check_statements(body, diagnostics);
                }
                ast::StmtKind::ClassDef { name, body, .. } => {
                    if !Self::is_pascal_case(name) {
                        println!("Found bad class name: {}", name);
                        diagnostics.push(Diagnostic {
                            level: DiagnosticLevel::Warning,
                            message: format!("Class '{}' should use PascalCase", name),
                            line: stmt.location.row(),
                            column: 1,
                            path: String::new(),
                        });
                    }
                    // Recurse into class body
                    self.check_statements(body, diagnostics);
                }
                ast::StmtKind::Assign { targets, value, .. } => {
                    for target in targets {
                        if let ast::ExprKind::Name { id, .. } = &target.node {
                            if !Self::is_valid_variable_name(id) {
                                println!("Found bad variable name: {}", id);
                                diagnostics.push(Diagnostic {
                                    level: DiagnosticLevel::Warning,
                                    message: format!(
                                        "Variable '{}' should use snake_case or be a proper constant",
                                        id
                                    ),
                                    line: stmt.location.row(),
                                    column: 1,
                                    path: String::new(),
                                });
                            }
                        }
                    }
                    // Recurse into the value expression
                    self.check_expr(value, diagnostics);
                }
                ast::StmtKind::If { test, body, orelse, .. } => {
                    self.check_expr(test, diagnostics);
                    self.check_statements(body, diagnostics);
                    self.check_statements(orelse, diagnostics);
                }
                ast::StmtKind::While { test, body, orelse, .. } => {
                    self.check_expr(test, diagnostics);
                    self.check_statements(body, diagnostics);
                    self.check_statements(orelse, diagnostics);
                }
                ast::StmtKind::For { target, iter, body, orelse, .. } => {
                    self.check_expr(target, diagnostics);
                    self.check_expr(iter, diagnostics);
                    self.check_statements(body, diagnostics);
                    self.check_statements(orelse, diagnostics);
                }
                ast::StmtKind::Expr { value, .. } => {
                    self.check_expr(value, diagnostics);
                }
                _ => {}
            }
        }
    }

    fn check_expr(&self, expr: &ast::Expr, diagnostics: &mut Vec<Diagnostic>) {
        match &expr.node {
            ast::ExprKind::Lambda { body, .. } => {
                self.check_expr(body, diagnostics);
            }
            ast::ExprKind::Call { func, args, keywords } => {
                self.check_expr(func, diagnostics);
                for arg in args {
                    self.check_expr(arg, diagnostics);
                }
                for keyword in keywords {
                    self.check_expr(&keyword.node.value, diagnostics);
                }
            }
            ast::ExprKind::IfExp { test, body, orelse } => {
                self.check_expr(test, diagnostics);
                self.check_expr(body, diagnostics);
                self.check_expr(orelse, diagnostics);
            }
            ast::ExprKind::BoolOp { values, .. } => {
                for value in values {
                    self.check_expr(value, diagnostics);
                }
            }
            ast::ExprKind::BinOp { left, right, .. } => {
                self.check_expr(left, diagnostics);
                self.check_expr(right, diagnostics);
            }
            // Add other expression kinds as needed
            _ => {}
        }
    }
}

impl super::Rule for NamingConventions {
    fn check(&self, ast: &ast::Suite, _source: &str) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        self.check_statements(ast, &mut diagnostics);
        println!("Total diagnostics: {}", diagnostics.len());
        Ok(diagnostics)
    }
}
