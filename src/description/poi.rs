use nalgebra::{Vector2, Vector3};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::{description::agent::Agent, util::{vector2_distance_f64, vector3_distance_f64}};

use super::rating::Rating;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type",rename_all = "camelCase")]
pub enum PointOfInterest {
    Standing(Location),
    Hand(Location)
}

impl PointOfInterest {

    pub fn new_standing(name: String, x: f64, y: f64, z: f64, variability: Option<Rating>, structure: Option<Rating>) -> Self {
        Self::Standing(Location::new(name, x, y, z, variability.unwrap_or(Rating::Medium), structure.unwrap_or(Rating::Medium)))
    }

    pub fn new_hand(name: String, x: f64, y: f64, z: f64, variability: Option<Rating>, structure: Option<Rating>) -> Self {
        Self::Hand(Location::new(name, x, y, z, variability.unwrap_or(Rating::Medium), structure.unwrap_or(Rating::Medium)))
    }

    pub fn is_standing(&self) -> bool {
        match self {
            PointOfInterest::Standing(_) => true,
            _ => false
        }
    }

    pub fn position(&self) -> Vector3<f64> {
        match self {
            PointOfInterest::Standing(location) => location.position,
            PointOfInterest::Hand(location) => location.position
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

    pub fn name(&self) -> String {
        match self {
            PointOfInterest::Standing(location) => location.name.clone(),
            PointOfInterest::Hand(location) => location.name.clone()
        }
    }

    pub fn variability(&self) -> Rating {
        match self {
            PointOfInterest::Standing(location) => location.variability.clone(),
            PointOfInterest::Hand(location) => location.variability.clone()
        }
    }

    pub fn structure(&self) -> Rating {
        match self {
            PointOfInterest::Standing(location) => location.structure.clone(),
            PointOfInterest::Hand(location) => location.structure.clone()
        }
    }

    pub fn reachability(&self, other: &PointOfInterest, agent: &Agent) -> bool {
        if (self.is_standing() && other.is_standing()) || (self.is_hand() && other.is_hand()) {
            return false;
        }
        let distance: f64 = (&self.position() - &other.position()).norm();
        match agent {
            Agent::Robot(robot_info) => {
                return distance <= robot_info.reach && distance >= robot_info.reach * 0.05;
            },
            Agent::Human(human_info) => {
                let mut offset_pos = self.position().clone();
                offset_pos.z += human_info.acromial_height;

                let self_pos = Vector2::new(self.position().x, self.position().y);
                let other_pos = Vector2::new(other.position().x, other.position().y);

                let horizontal_distance: f64 = (self_pos.clone() - other_pos.clone()).norm();
                let total_distance: f64 = (offset_pos.clone() - other.position().clone()).norm();
                
                // person can bend/reach down
                if other.position().z <= offset_pos.z {
                    return horizontal_distance <= human_info.reach;
                }
                // person is standing/can reach up
                return total_distance <= human_info.reach;
            }
        }
    }

    pub fn travelability(&self, other: &PointOfInterest, agent: &Agent) -> bool {
        if self.is_hand() || other.is_hand() {
            return false;
        }
        match agent {
            Agent::Robot(robot_info) => {
                return robot_info.mobile_speed > 0.0 && self.position().z - other.position().z <= 0.05;
            },
            Agent::Human(_human_info) => {
                return true;
            }
        }
    }

}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub id: Uuid,
    pub name: String,
    pub position: Vector3<f64>,
    pub shape: Shape,
    pub displacement: Vector3<f64>,
    pub variability: Rating,
    pub structure: Rating,

}

impl Location {
    pub fn new(name: String, x: f64, y: f64, z: f64, variability: Rating, structure: Rating) -> Self {
        Self { 
            id: Uuid::new_v4(), name, position: Vector3::new(x, y, z),
            shape: Shape::Ellipsoid, displacement: Vector3::new(0.0, 0.0, 0.0),
            variability, structure
        }
    }
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Shape {
    Ellipsoid,
    Cuboid
}