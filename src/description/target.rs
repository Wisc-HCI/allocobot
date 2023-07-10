use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type",rename_all = "camelCase")]
pub enum Target {
    Precursor {
        id: Uuid,
        name: String,
        size: f64,
        weight: f64,
        
    },
    Intermediate {
        id: Uuid,
        name: String,
        size: f64,
        weight: f64
    },
    Product {
        id: Uuid,
        name: String,
        size: f64,
        weight: f64
    },
    Reusable {
        id: Uuid,
        name: String,
        size: f64,
        weight: f64
    }
}

impl Target  {
    pub fn new_precursor(name: String, size: f64, weight: f64) -> Target  {
        Target::Precursor {
            id: Uuid::new_v4(),
            name,
            size,
            weight
        }
    }

    pub fn new_intermediate(name: String, size: f64, weight: f64) -> Target  {
        Target::Intermediate {
            id: Uuid::new_v4(),
            name,
            size,
            weight
        }
    }

    pub fn new_product(name: String, size: f64, weight: f64) -> Target  {
        Target::Product {
            id: Uuid::new_v4(),
            name,
            size,
            weight
        }
    }

    pub fn new_reusable(name: String, size: f64, weight: f64) -> Target  {
        Target::Reusable {
            id: Uuid::new_v4(),
            name,
            size,
            weight
        }
    }

    pub fn id(&self) -> Uuid {
        match self {
            Target::Precursor { id, .. } => id.clone(),
            Target::Intermediate { id, .. } => id.clone(),
            Target::Product { id, .. } => id.clone(),
            Target::Reusable { id, .. } => id.clone()
        }
    }

    pub fn name(&self) -> String {
        match self {
            Target::Precursor { name, .. } => name.clone(),
            Target::Intermediate { name, .. } => name.clone(),
            Target::Product { name, .. } => name.clone(),
            Target::Reusable { name, .. } => name.clone()
        }
    }
}

