use std::collections::HashMap;
use z3::{
    ast::{self, Ast},
    Context, DatatypeAccessor, DatatypeBuilder, DatatypeSort, Sort,
};

use crate::{agents::Agent, poi::PointOfInterest, tasks::TaskInfo};

pub enum StateProperty<'a> {
    Boolean {
        value: ast::Bool<'a>,
        modified: ast::Bool<'a>,
    },
    Token {
        value: HashMap<String, ast::Bool<'a>>,
        modified: ast::Bool<'a>,
        capacity: usize,
    },
}

impl<'a> StateProperty<'a> {
    pub fn new_boolean(ctx: &'a Context, name: &String) -> Self {
        StateProperty::Boolean {
            value: ast::Bool::new_const(ctx, name.clone()),
            modified: ast::Bool::new_const(ctx, format!("{}_modified", name)),
        }
    }

    pub fn new_token(
        ctx: &'a Context,
        name: String,
        options: &Vec<&String>,
        capacity: usize,
    ) -> Self {
        let mut value: HashMap<String, ast::Bool> = HashMap::new();
        for option in options.iter() {
            value.insert(
                format!("{}_{}", name, option),
                ast::Bool::new_const(ctx, format!("{}_{}", name, option)),
            );
        }
        StateProperty::Token {
            value,
            modified: ast::Bool::new_const(ctx, format!("{}_modified", name)),
            capacity,
        }
    }

    pub fn get_modified(&self) -> &ast::Bool {
        match self {
            Self::Boolean { modified, .. } => modified,
            Self::Token { modified, .. } => modified,
        }
    }

    // pub fn get_assertions(&self, previous: &StateProperty) -> Vec<ast::Bool> {
    //     let mut assertions: Vec<ast::Bool> = vec![];
    //     match (self, previous) {
    //         (
    //             StateProperty::Boolean { value: value1, ..},
    //             StateProperty::Boolean { value: value2, modified, ..}
    //         ) => {
    //             assertions.push(modified.not().implies(&value1._eq(value2)));
    //             assertions
    //         },
    //         (
    //             StateProperty::Token { value: value1, ..},
    //             StateProperty::Token { value: value2, ..}
    //         ) => {
    //             assertions
    //         },
    //         _ => {
    //             assertions
    //         }
    //     }
    // }
}

pub fn new_state_type<'ctx>(
    ctx: &'ctx Context,
    agents: &HashMap<String, &Agent>,
    pois: &HashMap<String, PointOfInterest>,
    tasks: &HashMap<String, &TaskInfo>,
) -> (DatatypeSort<'ctx>) {
    
    let mut z3_agents: HashMap<String, Z3Agent<'_>> = HashMap::new();
    let mut z3_tasks: HashMap<String, Z3Task<'_>> = HashMap::new();

    for (agent_id, agent) in self.agents.iter() {
        let z3_agent: Z3Agent;
        match &agent {
            Agent::Human(human_info) => z3_agent = Z3Agent::new_human(&self.ctx, human_info),
            Agent::Robot(robot_info) => z3_agent = Z3Agent::new_robot(&self.ctx, robot_info),
        }
        z3_agents.insert(agent_id.clone(), z3_agent);
    }

    for (task_id, task) in self.tasks.iter() {
        let agent: Option<&Z3Agent>;
        match &task.agent_id {
            Some(agent_id) => agent = z3_agents.get(agent_id),
            None => agent = None,
        };
        z3_tasks.insert(task_id.clone(), Z3Task::new(&self.ctx, task, agent));
    }

    let mut attributes = vec![];
    for agent_id in agents.keys() {
        for task_id in tasks.keys() {
            attributes.push((
                format!("{}_performs_{}", agent_id, task_id),
                DatatypeAccessor::Sort(Sort::bool(ctx)),
            ))
        }
        for poi_id in pois.keys() {}
    }
    let state = DatatypeBuilder::new(ctx, "State")
        .variant("default", vec![])
        .finish();
    (state)
}
