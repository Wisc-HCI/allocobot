// use z3::ast;
// use z3::{Context, Model};
use uuid::Uuid;
use crate::description::primitive::Primitive;

#[derive(Clone, Debug, PartialEq)]
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
        mobile: bool    // true/false
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
            mobile
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

    pub fn assess_cost(&self, _primitive: Primitive) -> f64 {
        0.0
    }

    pub fn assess_time(&self, _primitive: Primitive) -> f64 {
        0.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RobotInfo {
    pub id: Uuid,
    pub name: String,
    pub reach: f64,
    pub payload: f64,
    pub agility: f64,
    pub speed: f64,
    pub precision: f64,
    pub sensing: f64,
    pub mobile: bool
}

#[derive(Clone, Debug, PartialEq)]
pub struct HumanInfo {
    pub id: Uuid,
    pub name: String,
}