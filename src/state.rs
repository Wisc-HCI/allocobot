use std::collections::HashMap;

use z3::ast;
pub enum StateProperty<'a> {
    Boolean { value: ast::Bool<'a>, modified: ast::Bool<'a> },
    Token { value: HashMap<String,ast::Bool<'a>>, modified: ast::Bool<'a>, capacity: usize }
}