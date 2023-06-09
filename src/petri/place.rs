use uuid::Uuid;
use crate::petri::token::TokenSet;

#[derive(Clone, Debug, PartialEq)]
pub struct Place {
    pub id: Uuid,
    pub name: String,
    pub tokens: TokenSet,
    pub source_task: Option<Uuid>,
    pub target_id: Option<Uuid>,
}

impl Place {
    pub fn new(name: String, tokens: TokenSet, source_task: Option<Uuid>, target_id: Option<Uuid>) -> Self {
        Self { id: Uuid::new_v4(), name, tokens, source_task, target_id }
    }
}