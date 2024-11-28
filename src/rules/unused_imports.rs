use anyhow::Result;
use rustpython_parser::ast;
use crate::linter::Diagnostic;
use crate::linter::DiagnosticLevel;
use std::collections::HashSet;

pub struct UnusedImports;

impl UnusedImports {
    fn collect_used_names(&self, expr: &ast::Expr, used_names: &mut HashSet<String>) {
        match &expr.node {
            ast::ExprKind::Name { id, .. } => {
                used_names.insert(id.to_string());
            }
            ast::ExprKind::Attribute { value, attr, .. } => {
                // For attributes like sys.path, collect both sys and path
                self.collect_used_names(value, used_names);
                used_names.insert(attr.to_string());
                // Also collect the full path (e.g., "sys.path")
                if let ast::ExprKind::Name { id, .. } = &value.node {
                    used_names.insert(format!("{}.{}", id, attr));
                }
            }
            ast::ExprKind::Call { func, args, .. } => {
                self.collect_used_names(func, used_names);
                for arg in args {
                    self.collect_used_names(arg, used_names);
                }
            }
            ast::ExprKind::BinOp { left, right, .. } => {
                self.collect_used_names(left, used_names);
                self.collect_used_names(right, used_names);
            }
            ast::ExprKind::Subscript { value, slice, .. } => {
                self.collect_used_names(value, used_names);
                self.collect_used_names(slice, used_names);
            }
            _ => {}
        }
    }

    fn collect_used_names_from_stmt(&self, stmt: &ast::Stmt, used_names: &mut HashSet<String>) {
        match &stmt.node {
            ast::StmtKind::FunctionDef { body, args, .. } => {
                // Process function arguments
                for arg in &args.args {
                    if let Some(annotation) = &arg.node.annotation {
                        self.collect_used_names(annotation, used_names);
                    }
                }
                // Process function body
                for stmt in body {
                    self.collect_used_names_from_stmt(stmt, used_names);
                }
            }
            ast::StmtKind::Assign { targets, value, .. } => {
                for target in targets {
                    self.collect_used_names(target, used_names);
                }
                self.collect_used_names(value, used_names);
            }
            ast::StmtKind::AnnAssign { target, annotation, value, .. } => {
                self.collect_used_names(target, used_names);
                self.collect_used_names(annotation, used_names);
                if let Some(val) = value {
                    self.collect_used_names(val, used_names);
                }
            }
            ast::StmtKind::Return { value, .. } => {
                if let Some(val) = value {
                    self.collect_used_names(val, used_names);
                }
            }
            ast::StmtKind::Expr { value } => {
                self.collect_used_names(value, used_names);
            }
            _ => {}
        }
    }
}

impl super::Rule for UnusedImports {
    fn check(&self, ast: &ast::Suite, _source: &str) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let mut used_names = HashSet::new();
        let mut defined_imports = Vec::new();

        // First pass: collect all imports
        for stmt in ast.iter() {
            match &stmt.node {
                ast::StmtKind::Import { names } => {
                    for alias in names {
                        let import_name = match &alias.node.asname {
                            Some(asname) => (alias.node.name.to_string(), stmt.location.row(), Some(asname.to_string())),
                            None => (alias.node.name.to_string(), stmt.location.row(), None),
                        };
                        defined_imports.push(import_name);
                    }
                }
                ast::StmtKind::ImportFrom { module: Some(_module), names, .. } => {
                    for alias in names {
                        let import_name = match &alias.node.asname {
                            Some(asname) => (alias.node.name.to_string(), stmt.location.row(), Some(asname.to_string())),
                            None => (alias.node.name.to_string(), stmt.location.row(), None),
                        };
                        defined_imports.push(import_name);
                    }
                }
                _ => {}
            }
        }

        // Second pass: collect used names
        for stmt in ast.iter() {
            self.collect_used_names_from_stmt(stmt, &mut used_names);
        }

        // Check for unused imports
        for (name, line, asname) in defined_imports {
            let is_used = if let Some(ref alias) = asname {
                used_names.contains(alias)
            } else {
                let name_parts: Vec<&str> = name.split('.').collect();
                let base_name = name_parts[0];
                used_names.contains(&name) || 
                used_names.contains(base_name) ||
                used_names.iter().any(|used| used.starts_with(&format!("{}.", base_name)))
            };

            if !is_used {
                let display_name = if let Some(alias) = asname {
                    format!("{} as {}", name, alias)
                } else {
                    name
                };
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Warning,
                    message: format!("Unused import '{}'", display_name),
                    line,
                    column: 1,
                    path: String::new(),
                });
            }
        }

        Ok(diagnostics)
    }
}
