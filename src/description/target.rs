use crate::description::agent::Agent;
use enum_tag::EnumTag;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::description::rating::Rating;

use super::poi::PointOfInterest;

#[derive(Clone, Debug, PartialEq, EnumTag, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Target {
    Precursor {
        id: Uuid,
        name: String,
        size: f64,
        weight: f64,
        symmetry: Rating,
        pois: Vec<Uuid>,
        value: f64,
    },
    Intermediate {
        id: Uuid,
        name: String,
        size: f64,
        weight: f64,
        symmetry: Rating,
        pois: Vec<Uuid>,
        value: f64,
    },
    Product {
        id: Uuid,
        name: String,
        size: f64,
        weight: f64,
        symmetry: Rating,
        pois: Vec<Uuid>,
        value: f64,
    },
    Reusable {
        id: Uuid,
        name: String,
        size: f64,
        weight: f64,
        symmetry: Rating,
        pois: Vec<Uuid>,
        value: f64,
    },
}

impl Target {
    pub fn new_precursor(
        name: String,
        size: f64,
        weight: f64,
        symmetry: Rating,
        pois: Vec<Uuid>,
        value: f64,
    ) -> Target {
        Target::Precursor {
            id: Uuid::new_v4(),
            name,
            size,
            weight,
            symmetry,
            pois,
            value,
        }
    }

    pub fn new_intermediate(
        name: String,
        size: f64,
        weight: f64,
        symmetry: Rating,
        pois: Vec<Uuid>,
    ) -> Target {
        Target::Intermediate {
            id: Uuid::new_v4(),
            name,
            size,
            weight,
            symmetry,
            pois,
            value: 0.0,
        }
    }

    pub fn new_product(
        name: String,
        size: f64,
        weight: f64,
        symmetry: Rating,
        pois: Vec<Uuid>,
        value: f64,
    ) -> Target {
        Target::Product {
            id: Uuid::new_v4(),
            name,
            size,
            weight,
            symmetry,
            pois,
            value,
        }
    }

    pub fn new_reusable(
        name: String,
        size: f64,
        weight: f64,
        symmetry: Rating,
        pois: Vec<Uuid>,
    ) -> Target {
        Target::Reusable {
            id: Uuid::new_v4(),
            name,
            size,
            weight,
            symmetry,
            pois,
            value: 0.0,
        }
    }

    pub fn carryable(&self, agent: &Agent) -> bool {
        match agent {
            Agent::Human(_) => true,
            Agent::Robot(robot_info) => match self {
                Target::Precursor { weight, .. } => robot_info.payload >= *weight,
                Target::Intermediate { weight, .. } => robot_info.payload >= *weight,
                Target::Product { weight, .. } => robot_info.payload >= *weight,
                Target::Reusable { weight, .. } => robot_info.payload >= *weight,
            },
        }
    }

    pub fn id(&self) -> Uuid {
        match self {
            Target::Precursor { id, .. } => id.clone(),
            Target::Intermediate { id, .. } => id.clone(),
            Target::Product { id, .. } => id.clone(),
            Target::Reusable { id, .. } => id.clone(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Target::Precursor { name, .. } => name.clone(),
            Target::Intermediate { name, .. } => name.clone(),
            Target::Product { name, .. } => name.clone(),
            Target::Reusable { name, .. } => name.clone(),
        }
    }

    pub fn size(&self) -> f64 {
        match self {
            Target::Precursor { size, .. } => *size,
            Target::Intermediate { size, .. } => *size,
            Target::Product { size, .. } => *size,
            Target::Reusable { size, .. } => *size,
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            Target::Precursor { weight, .. } => *weight * 9.81,
            Target::Intermediate { weight, .. } => *weight * 9.81,
            Target::Product { weight, .. } => *weight * 9.81,
            Target::Reusable { weight, .. } => *weight * 9.81,
        }
    }

    pub fn pois(&self) -> Vec<Uuid> {
        match self {
            Target::Precursor { pois, .. } => pois.clone(),
            Target::Intermediate { pois, .. } => pois.clone(),
            Target::Product { pois, .. } => pois.clone(),
            Target::Reusable { pois, .. } => pois.clone(),
        }
    }

    pub fn value(&self) -> f64 {
        match self {
            Target::Precursor { value, .. } => *value,
            Target::Intermediate { value, .. } => *value,
            Target::Product { value, .. } => *value,
            Target::Reusable { value, .. } => *value,
        }
    }

    pub fn symmetry(&self) -> Rating {
        match self {
            Target::Precursor { symmetry, .. } => symmetry.clone(),
            Target::Intermediate { symmetry, .. } => symmetry.clone(),
            Target::Product { symmetry, .. } => symmetry.clone(),
            Target::Reusable { symmetry, .. } => symmetry.clone(),
        }
    }

    pub fn add_point_of_interest(&mut self, poi: &Uuid) {
        match self {
            Target::Precursor { pois, .. } => pois.push(*poi),
            Target::Intermediate { pois, .. } => pois.push(*poi),
            Target::Product { pois, .. } => pois.push(*poi),
            Target::Reusable { pois, .. } => pois.push(*poi),
        }
    }
}

pub type TargetTag = <Target as EnumTag>::Tag;
