mod unused_imports;
mod line_length;
mod naming_conventions;

use anyhow::Result;
use rustpython_parser::ast;
use crate::linter::Diagnostic;

pub use unused_imports::UnusedImports;
pub use line_length::LineLength;
pub use naming_conventions::NamingConventions;

pub trait Rule: Send + Sync {
    fn check(&self, ast: &ast::Suite, source: &str) -> Result<Vec<Diagnostic>>;
}

pub fn get_default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(UnusedImports),
        Box::new(LineLength::new(88)), // Default to 88 characters (black formatter default)
        Box::new(NamingConventions),
    ]
}
