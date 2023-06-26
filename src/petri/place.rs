use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::petri::token::TokenSet;
use crate::petri::data::{Data,data_subset};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub id: Uuid,
    pub name: String,
    pub tokens: TokenSet,
    pub meta_data: Vec<Data>,
}

impl Place {
    pub fn new(name: String, tokens: TokenSet, meta_data: Vec<Data>) -> Self {
        Self { id: Uuid::new_v4(), name, tokens, meta_data }
    }

    pub fn has_data(&self, meta_data: &Vec<Data>, fuzzy: bool) -> bool {
        data_subset(&self.meta_data, meta_data, fuzzy)
    }
}