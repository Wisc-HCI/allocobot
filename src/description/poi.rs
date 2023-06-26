use nalgebra::Vector3;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type",rename_all = "camelCase")]
pub enum PointOfInterest {
    Standing(Location),
    Hand(Location)
}

impl PointOfInterest {

    pub fn new_standing(name: String, x: f64, y: f64, z: f64) -> Self {
        Self::Standing(Location::new(name, x, y, z))
    }

    pub fn new_hand(name: String, x: f64, y: f64, z: f64) -> Self {
        Self::Hand(Location::new(name, x, y, z))
    }

    pub fn is_standing(&self) -> bool {
        match self {
            PointOfInterest::Standing(_) => true,
            _ => false
        }
    }

    pub fn is_hand(&self) -> bool {
        match self {
            PointOfInterest::Hand(_) => true,
            _ => false
        }
    }

    pub fn id(&self) -> Uuid {
        match self {
            PointOfInterest::Standing(location) => location.id.clone(),
            PointOfInterest::Hand(location) => location.id.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub id: Uuid,
    pub name: String,
    pub position: Vector3<f64>
}

impl Location {
    pub fn new(name: String, x: f64, y: f64, z: f64) -> Self {
        Self { id: Uuid::new_v4(), name, position: Vector3::new(x, y, z)}
    }
}