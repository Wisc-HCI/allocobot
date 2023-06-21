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

// pub enum Z3Agent<'a> {
//     Robot(Z3RobotInfo<'a>),
//     Human(Z3HumanInfo<'a>),
// }

// impl<'a> Z3Agent<'a> {
//     pub fn new_robot(ctx: &'a Context, robot_info: &RobotInfo) -> Self {
//         return Z3Agent::Robot(Z3RobotInfo {
//             robot_info: robot_info.clone(),
//             z3_id: ast::Int::new_const(ctx, robot_info.id.clone()),
//         });
//     }

//     pub fn new_human(ctx: &'a Context, human_info: &HumanInfo) -> Self {
//         return Z3Agent::Human(Z3HumanInfo {
//             human_info: human_info.clone(),
//             z3_id: ast::Int::new_const(ctx, human_info.id.clone()),
//         });
//     }

//     pub fn get_id(&self) -> String {
//         match self {
//             Z3Agent::Robot(robot_info) => return robot_info.robot_info.id.clone(),
//             Z3Agent::Human(human_info) => return human_info.human_info.id.clone(),
//         }
//     }

//     pub fn get_name(&self) -> String {
//         match self {
//             Z3Agent::Robot(robot_info) => return robot_info.robot_info.name.clone(),
//             Z3Agent::Human(human_info) => return human_info.human_info.name.clone(),
//         }
//     }

//     pub fn get_z3_id(&self) -> &ast::Int {
//         match self {
//             Z3Agent::Robot(robot_info) => return &robot_info.z3_id,
//             Z3Agent::Human(human_info) => return &human_info.z3_id,
//         }
//     }

//     pub fn get_agent_code(&self, model: &Model) -> i64 {
//         return model
//             .eval(self.get_z3_id(), true)
//             .map_or(0, |z3inner| z3inner.as_i64().unwrap_or(0));
//     }
// }

// pub struct Z3RobotInfo<'a> {
//     pub robot_info: RobotInfo,
//     pub z3_id: ast::Int<'a>,
// }

// pub struct Z3HumanInfo<'a> {
//     pub human_info: HumanInfo,
//     pub z3_id: ast::Int<'a>,
// }
