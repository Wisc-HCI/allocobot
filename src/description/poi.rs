use nalgebra::Vector3;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::description::agent::Agent;

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

    pub fn reachability(&self, other: &PointOfInterest, agent: &Agent) -> bool {
        if (self.is_standing() && other.is_standing()) || (self.is_hand() && other.is_hand()) {
            return false;
        }
        let distance: f64 = (&self.position() - &other.position()).norm();
        match agent {
            Agent::Robot(robot_info) => {
                return distance <= robot_info.reach && distance >= robot_info.reach * 0.05;
            },
            Agent::Human(_human_info) => {
                return true;
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

    pub fn movement_cost(&self, other: &PointOfInterest, agent: &Agent) -> f64 {
        match (self, other) {
            // Compute as a travel cost for standing/standing queries
            (PointOfInterest::Standing(_self_location), PointOfInterest::Standing(_other_location)) => {
                match agent {
                    Agent::Robot(_robot_info) => {
                        return 0.0;
                    },
                    Agent::Human(_human_info) => {
                        return 0.0;
                    }
                }
            },
            // Compute as a hand movement cost for hand/hand queries
            (PointOfInterest::Hand(_), PointOfInterest::Hand(_)) => {
                match agent {
                    Agent::Robot(_robot_info) => {
                        return 0.0;
                    },
                    Agent::Human(_human_info) => {
                        return 0.0;
                    }
                }
            },
            // Consider this a query into the cost of statically holding a hand at a standing location
            _ => {
                match agent {
                    Agent::Robot(_robot_info) => {
                        return 0.0;
                    },
                    Agent::Human(_human_info) => {
                        return 0.0;
                    }
                }
            }
        }
    }

    pub fn movement_time(&self, other: &PointOfInterest, agent: &Agent, _precision: f64) -> f64 {
        let distance = (&self.position() - &other.position()).norm();
        match (self, other) {
            // Compute as a travel time for standing/standing queries
            (PointOfInterest::Standing(_self_location), PointOfInterest::Standing(_other_location)) => {
                match agent {
                    Agent::Robot(robot_info) => {
                        if robot_info.mobile_speed > 0.0 {
                            return distance / robot_info.mobile_speed;
                        } else {
                            return 0.0;
                        }
                    },
                    Agent::Human(_human_info) => {
                        return 0.0;
                    }
                }
            },
            // Compute as a hand movement time for hand/hand queries
            (PointOfInterest::Hand(_), PointOfInterest::Hand(_)) => {
                match agent {
                    Agent::Robot(robot_info) => {
                        return distance / robot_info.speed;
                    },
                    Agent::Human(_human_info) => {
                        return 0.0;
                    }
                }
            },
            // Can't move from standing to hand or vice versa, so return 0.0
            _ => {
                return 0.0;
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
    pub variability: f64,
    pub structure: f64,

}

impl Location {
    pub fn new(name: String, x: f64, y: f64, z: f64) -> Self {
        Self { 
            id: Uuid::new_v4(), name, position: Vector3::new(x, y, z),
            shape: Shape::Ellipsoid, displacement: Vector3::new(0.0, 0.0, 0.0),
            variability: 0.0, structure: 0.0
        }
    }
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Shape {
    Ellipsoid,
    Cuboid
}