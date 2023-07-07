use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TokenSet {
    Infinite,
    Sink,
    Finite,
}

impl Default for TokenSet {
    fn default() -> Self {
        TokenSet::Finite
    }
}