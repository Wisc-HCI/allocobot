use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Weights {
    pub ergonomic: f64,
    pub monetary: f64
}

impl Default for Weights {
    fn default() -> Self {
        Weights {
            ergonomic: 1.0,
            monetary: 1.0
        }
    }
}