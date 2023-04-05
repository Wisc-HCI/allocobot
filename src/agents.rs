use z3::ast;
use z3::{Context, Model};

#[derive(Clone, Debug)]
pub enum Agent {
    Robot(RobotInfo),
    Human(HumanInfo),
}

impl Agent {
    pub fn new_robot(id: String, name: String) -> Self {
        return Agent::Robot(RobotInfo {
            id: id.clone(),
            name,
        });
    }

    pub fn new_human(id: String, name: String) -> Self {
        return Agent::Human(HumanInfo {
            id: id.clone(),
            name,
        });
    }

    pub fn get_id(&self) -> String {
        match self {
            Agent::Robot(robot_info) => return robot_info.id.clone(),
            Agent::Human(human_info) => return human_info.id.clone(),
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            Agent::Robot(robot_info) => return robot_info.name.clone(),
            Agent::Human(human_info) => return human_info.name.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RobotInfo {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct HumanInfo {
    pub id: String,
    pub name: String,
}

pub enum Z3Agent<'a> {
    Robot(Z3RobotInfo<'a>),
    Human(Z3HumanInfo<'a>),
}

impl<'a> Z3Agent<'a> {
    pub fn new_robot(ctx: &'a Context, robot_info: &RobotInfo) -> Self {
        return Z3Agent::Robot(Z3RobotInfo {
            robot_info: robot_info.clone(),
            z3_id: ast::Int::new_const(ctx, robot_info.id.clone()),
        });
    }

    pub fn new_human(ctx: &'a Context, human_info: &HumanInfo) -> Self {
        return Z3Agent::Human(Z3HumanInfo {
            human_info: human_info.clone(),
            z3_id: ast::Int::new_const(ctx, human_info.id.clone()),
        });
    }

    pub fn get_id(&self) -> String {
        match self {
            Z3Agent::Robot(robot_info) => return robot_info.robot_info.id.clone(),
            Z3Agent::Human(human_info) => return human_info.human_info.id.clone(),
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            Z3Agent::Robot(robot_info) => return robot_info.robot_info.name.clone(),
            Z3Agent::Human(human_info) => return human_info.human_info.name.clone(),
        }
    }

    pub fn get_z3_id(&self) -> &ast::Int {
        match self {
            Z3Agent::Robot(robot_info) => return &robot_info.z3_id,
            Z3Agent::Human(human_info) => return &human_info.z3_id,
        }
    }

    pub fn get_agent_code(&self, model: &Model) -> i64 {
        return model
            .eval(self.get_z3_id(), true)
            .map_or(0, |z3inner| z3inner.as_i64().unwrap_or(0));
    }
}

pub struct Z3RobotInfo<'a> {
    pub robot_info: RobotInfo,
    pub z3_id: ast::Int<'a>,
}

pub struct Z3HumanInfo<'a> {
    pub human_info: HumanInfo,
    pub z3_id: ast::Int<'a>,
}
