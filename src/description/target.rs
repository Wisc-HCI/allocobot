use uuid::Uuid;
use serde::{Serialize, Deserialize};
use enum_tag::EnumTag;
use crate::description::agent::Agent;

#[derive(Clone, Debug, PartialEq, EnumTag, Serialize, Deserialize)]
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

    pub fn carryable(&self, agent: &Agent) -> bool {
        match agent {
            Agent::Human(_) => true,
            Agent::Robot(robot_info) => {
                match self {
                    Target::Precursor { weight, .. } => robot_info.payload >= *weight,
                    Target::Intermediate { weight, .. } => robot_info.payload >= *weight,
                    Target::Product { weight, .. } => robot_info.payload >= *weight,
                    Target::Reusable { weight, .. } => robot_info.payload >= *weight
                }
            }
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

pub type TargetTag = <Target as EnumTag>::Tag;