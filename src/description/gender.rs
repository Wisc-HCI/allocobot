use std::cmp::{Ord, PartialEq, Eq, PartialOrd};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Ord, PartialOrd, Eq)]
#[serde(tag = "type",rename_all = "camelCase")]
pub enum Gender {
    Female,
    Male
}
