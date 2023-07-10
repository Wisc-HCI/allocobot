use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::petri::token::TokenSet;
use crate::petri::data::{Data, data_query, Query};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
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

    pub fn has_data(&self, query_vec: &Vec<Query>) -> bool {
        data_query(&self.meta_data, query_vec)
    }
}