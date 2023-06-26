use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type",rename_all = "camelCase")]
pub enum TargetVariant {
    RawMaterial,
    Intermediate,
    Product,
    Singular
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Target {
    pub id: Uuid,
    // pub target_variant: TargetVariant,
    pub name: String,
    pub size: f64,
    pub weight: f64
}

impl Target  {
    pub fn new(name: String, size: f64, weight: f64) -> Target  {
        Target {
            id: Uuid::new_v4(),
            name,
            size,
            weight
        }
    }
}

