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
            ergonomic: 0.5,
            monetary: 0.5
        }
    }
}