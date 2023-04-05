use std::collections::HashMap;
use z3::ast;
use z3::Context;
use crate::tasks::Z3Task;
use crate::agents::Z3Agent;

pub struct Timeline<'a> {
    pub times: Vec<ast::Int<'a>>,
    pub busy_grid: HashMap<String, HashMap<String, Vec<ast::Bool<'a>>>>,
    pub assertions: Vec<ast::Bool<'a>>,
}

impl<'a> Timeline<'a> {
    pub fn new(ctx: &'a Context, tasks: &HashMap<String,Z3Task>, agents: &HashMap<String,Z3Agent>) -> Self {
        let times: Vec<ast::Int> = (0..tasks.len() * 2)
            .map(|i| ast::Int::new_const(ctx, format!("t{}", i)))
            .collect();
        let mut busy_grid: HashMap<String, HashMap<String, Vec<ast::Bool<'a>>>> = HashMap::new();

        for agent in agents.values() {
            let mut agent_busy_grid: HashMap<String, Vec<ast::Bool<'a>>> = HashMap::new();
            for task in tasks.values() {
                agent_busy_grid.insert(
                    task.task_info.id.clone(),
                    (0..tasks.len() * 3)
                        .map(|i| {
                            ast::Bool::new_const(
                                ctx,
                                format!("b_{}_{}_{}", agent.get_id(), task.task_info.id, i),
                            )
                        })
                        .collect(),
                );
            }
            busy_grid.insert(agent.get_id(), agent_busy_grid);
        }

        let mut assertions: Vec<ast::Bool> = Vec::new();

        for time_idx in 0..times.len() {
            for agent in agents.values() {
                let mut agent_does_task: Vec<&ast::Bool> = Vec::new();
                for task in tasks.values() {
                    agent_does_task.push(
                        &busy_grid
                            .get(&agent.get_id())
                            .unwrap()
                            .get(&task.task_info.id)
                            .unwrap()[time_idx],
                    );
                }
                let agent_busy_limit: ast::Bool = ast::Bool::pb_le(
                    ctx,
                    &agent_does_task
                        .iter()
                        .map(|t| (&t as &ast::Bool, 1 as i32))
                        .collect::<Vec<(&ast::Bool, i32)>>()
                        .as_slice(),
                    1,
                );
                assertions.push(agent_busy_limit);
            }
        }

        return Self {
            times,
            busy_grid,
            assertions,
        };
    }
}
