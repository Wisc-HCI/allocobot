use std::collections::HashMap;

use z3::ast;
use z3::{Model,Context};
use crate::agents::Z3Agent;
use crate::primitives::Primitive;
use crate::timeline::Timeline;
use z3::ast::Ast;

#[derive(Clone,Debug)]
pub struct TaskInfo {
    pub id: String,
    pub primitives: Vec<Primitive>,
    pub agent_id: Option<String>,
    pub duration: i64,
    pub task_dependencies: Vec<String>
}

impl TaskInfo {
    pub fn new(id: String, primitives: Vec<Primitive>, agent_id: Option<String>, duration: i64, task_dependencies: Vec<String>) -> Self {
        Self { id, primitives, agent_id, duration, task_dependencies}
    }
}

#[derive(Clone,Debug)]
pub struct AllocatedTask {
    pub id: String,
    pub primitives: Vec<Primitive>,
    pub agent_id: String,
    pub start_time: i64,
    pub end_time: i64,
    pub task_dependencies: Vec<String>
}

pub struct Z3Task<'a> {
    pub task_info: TaskInfo,
    pub agent: Option<&'a Z3Agent<'a>>,
    pub z3_agent: ast::Int<'a>,
    pub z3_start_time: ast::Int<'a>,
    pub z3_end_time: ast::Int<'a>,
    pub z3_duration: ast::Int<'a>,
}

impl<'a> Z3Task<'a> {
    pub fn new(ctx: &'a Context, task_info: &TaskInfo, agent: Option<&'a Z3Agent>) -> Self {
        
        return Self {
            task_info:task_info.clone(),
            agent,
            z3_agent: ast::Int::new_const(ctx, format!("agent-{}", task_info.id.clone())),
            z3_start_time: ast::Int::new_const(ctx, format!("start-{}", task_info.id.clone())),
            z3_end_time: ast::Int::new_const(ctx, format!("end-{}", task_info.id.clone())),
            z3_duration: ast::Int::from_i64(ctx, task_info.duration.clone()),
        };
    }

    pub fn get_assertions(
        &'a self,
        ctx: &'a Context,
        timeline: &Timeline<'a>,
        agents: &'a HashMap<String,Z3Agent<'a>>,
        tasks: &'a HashMap<String,Z3Task<'a>>,
    ) -> Vec<ast::Bool> {
        let mut assertions: Vec<ast::Bool> = Vec::new();

        // If an agent is defined, require that the z3_agent matches
        match self.agent {
            Some(agent) => {
                // Require that the agent match exactly
                assertions.push(agent.get_z3_id()._eq(&self.z3_agent));
            }
            None => {}
        }

        // Require that one of the available agents is the allocated agents
        let mut agent_matches: Vec<ast::Bool> = Vec::new();
        for agent in agents.values() {
            agent_matches.push(agent.get_z3_id()._eq(&self.z3_agent))
        }
        let assigned_is_agent: ast::Bool = ast::Bool::pb_eq(
            ctx,
            &agent_matches
                .iter()
                .map(|t| (&t as &ast::Bool, 1 as i32))
                .collect::<Vec<(&ast::Bool, i32)>>()
                .as_slice(),
            1,
        );
        assertions.push(assigned_is_agent);

        // Require that the end time is start + duration
        let start_plus_duration_equals_end = self.z3_end_time._eq(&ast::Int::add(
            ctx,
            &[&self.z3_start_time, &self.z3_duration],
        ));

        // Require that start time and end time appear in the timeline;
        // // Define a set of matching bool statements
        let mut start_matches: Vec<ast::Bool> = Vec::new();
        let mut end_matches: Vec<ast::Bool> = Vec::new();
        // let mut busy_rules: Vec<ast::Bool> = Vec::new();
        for event in &timeline.times {
            start_matches.push(event._eq(&self.z3_start_time));
            end_matches.push(event._eq(&self.z3_end_time));
        }

        let start_time_appears: ast::Bool = ast::Bool::pb_eq(
            ctx,
            &start_matches
                .iter()
                .map(|t| (&t as &ast::Bool, 1 as i32))
                .collect::<Vec<(&ast::Bool, i32)>>()
                .as_slice(),
            1,
        );

        let end_time_appears: ast::Bool = ast::Bool::pb_eq(
            ctx,
            &end_matches
                .iter()
                .map(|t| (&t as &ast::Bool, 1 as i32) as (&ast::Bool, i32))
                .collect::<Vec<(&ast::Bool, i32)>>()
                .as_slice(),
            1,
        );

        // Require that the start-time is after the end-time of any task dependencies
        for task_dependency in self.task_info.task_dependencies.iter() {
            match tasks.get(task_dependency) {
                Some(dep) => assertions.push(self.z3_start_time.gt(&dep.z3_end_time)),
                None => {
                    println!("Task Dependency {:?} not found!",task_dependency);
                }
            }
        }

        // For each agent, if assigned, then the robot is busy between start and end
        for agent in agents.values() {
            let agent_is_assigned: ast::Bool = self.z3_agent._eq(&agent.get_z3_id());
            let agent_task_busy_rules = timeline
                .times
                .iter()
                .enumerate()
                .map(|(time_idx, time)| {
                    ast::Bool::and(
                        ctx,
                        &[&time.ge(&self.z3_start_time), &time.le(&self.z3_end_time)],
                    )
                    .implies(
                        &timeline
                            .busy_grid
                            .get(&agent.get_id())
                            .unwrap()
                            .get(&self.task_info.id)
                            .unwrap()[time_idx],
                    )
                })
                .collect::<Vec<ast::Bool>>();
            assertions.push(
                agent_is_assigned.implies(&ast::Bool::and(
                    ctx,
                    &agent_task_busy_rules
                        .iter()
                        .collect::<Vec<&ast::Bool>>()
                        .as_slice(),
                )),
            );
        }

        assertions.push(start_plus_duration_equals_end);

        assertions.push(start_time_appears);

        assertions.push(end_time_appears);

        return assertions;
    }

    pub fn get_final_times(&self, model: &Model) -> (i64, i64) {
        (
            model
                .eval(&self.z3_start_time, true)
                .map_or(9999, |z3inner| z3inner.as_i64().unwrap_or(9999)),
            model
                .eval(&self.z3_end_time, true)
                .map_or(9999, |z3inner| z3inner.as_i64().unwrap_or(9999)),
        )
    }

    pub fn get_agent_code(&self, model: &Model) -> i64 {
        return model
            .eval(&self.z3_agent, true)
            .map_or(9999, |z3inner| z3inner.as_i64().unwrap_or(9999));
    }
}