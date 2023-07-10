use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type",rename_all = "camelCase")]
pub enum Agent {
    Robot(RobotInfo),
    Human(HumanInfo),
}

impl Agent {
    pub fn new_robot(
        name: String,
        reach: f64,     // meters
        payload: f64,   // kg
        agility: f64,   // rating 0-1
        speed: f64,     // m/s
        precision: f64, // m (repeatability)
        sensing: f64,   // rating 0-1
        mobile_speed: f64    // true/false
    ) -> Self {
        return Agent::Robot(RobotInfo {
            id: Uuid::new_v4(),
            name,
            reach,
            payload,
            agility,
            speed,
            precision,
            sensing,
            mobile_speed
        });
    }

    pub fn new_human(name: String) -> Self {
        return Agent::Human(HumanInfo {
            id: Uuid::new_v4(),
            name,
        });
    }

    pub fn id(&self) -> Uuid {
        match self {
            Agent::Robot(robot_info) => return robot_info.id.clone(),
            Agent::Human(human_info) => return human_info.id.clone(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Agent::Robot(robot_info) => return robot_info.name.clone(),
            Agent::Human(human_info) => return human_info.name.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RobotInfo {
    pub id: Uuid,
    pub name: String,
    pub reach: f64,
    pub payload: f64,
    pub agility: f64,
    pub speed: f64,
    pub precision: f64,
    pub sensing: f64,
    pub mobile_speed: f64
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HumanInfo {
    pub id: Uuid,
    pub name: String,
}