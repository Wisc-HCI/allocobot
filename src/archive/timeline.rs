use crate::agents::Z3Agent;
use crate::state::StateProperty;
use crate::tasks::Z3Task;
use std::collections::HashMap;
use z3::ast;
use z3::Context;

pub struct Timeline<'a> {
    pub times: Vec<ast::Int<'a>>,
    pub states: Vec<HashMap<String, StateProperty<'a>>>,
    pub busy_grid: HashMap<String, HashMap<String, Vec<ast::Bool<'a>>>>,
    pub assertions: Vec<ast::Bool<'a>>,
}

impl<'a> Timeline<'a> {
    pub fn new(
        ctx: &'a Context,
        tasks: &HashMap<String, Z3Task>,
        agents: &HashMap<String, Z3Agent>,
    ) -> Self {
        let mut assertions: Vec<ast::Bool> = Vec::new();

        let times: Vec<ast::Int> = (0..tasks.len() * 2)
            .map(|i| ast::Int::new_const(ctx, format!("t{}", i)))
            .collect();
        let mut states: Vec<HashMap<String, StateProperty<'a>>> =
            (0..tasks.len() * 2).map(|_| HashMap::new()).collect();

        for state in states.iter_mut() {
            for task in tasks.values() {
                let mut task_assignment: HashMap<String, ast::Bool<'a>> = HashMap::new();
                for agent in agents.values() {
                    task_assignment.insert(format!("b_{}_{}", agent.get_id(), task.task_info.id), ast::Bool::new_const(
                        ctx,
                        format!("b_{}_{}", agent.get_id(), task.task_info.id),
                    ));
                }
                state.insert(
                    format!("{}_agent", task.task_info.id),
                    StateProperty::Token {
                        value: task_assignment,
                        modified: ast::Bool::new_const(
                            ctx,
                            format!("{}_agent_modified", task.task_info.id),
                        ),
                        capacity: 1,
                    },
                );
            }
        }

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
            states,
            busy_grid,
            assertions,
        };
    }
}
