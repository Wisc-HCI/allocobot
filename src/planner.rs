use std::collections::HashMap;

use crate::tasks::{AllocatedTask, TaskInfo};
use crate::{agents::Agent, agents::Z3Agent, tasks::Z3Task, timeline::Timeline};
use z3::ast;
use z3::ast::Ast;
use z3::{Config, Context, Model, Optimize, SatResult};

pub struct Planner {
    pub tasks: HashMap<String, TaskInfo>,
    pub agents: HashMap<String, Agent>,
    pub ctx: Context,
}

impl Planner {
    pub fn new(tasks: &HashMap<String, TaskInfo>, agents: &HashMap<String, Agent>) -> Self {
        let ctx: Context = Context::new(&Config::default());
        return Self { tasks: tasks.clone(), agents: agents.clone(), ctx };
    }

    pub fn plan(&mut self) -> Result<HashMap<String,AllocatedTask>,String> {
        // Convert the non-z3 variants to their z3 forms.
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

        let timeline: Timeline = Timeline::new(&self.ctx, &z3_tasks, &z3_agents);
        let optimizer: Optimize = Optimize::new(&self.ctx);

        for assertion in &timeline.assertions {
            optimizer.assert(assertion)
        }

        for task in z3_tasks.values() {
            for assertion in task.get_assertions(&self.ctx, &timeline, &z3_agents).iter() {
                optimizer.assert(assertion)
            }
        }

        // Require that the first event is zero
        optimizer.assert(&timeline.times[0]._eq(&ast::Int::from_i64(&self.ctx, 0)));

        // Each time in the timeline must be greater than the one before it
        for event_idx in 0..(timeline.times.len() - 1) {
            optimizer.assert(&timeline.times[event_idx].lt(&timeline.times[event_idx + 1]))
        }

        // Require that the agent markers are distinct
        for (i, agent1) in z3_agents.values().enumerate() {
            for (j, agent2) in z3_agents.values().enumerate() {
                if i < j {
                    optimizer.assert(&agent1.get_z3_id()._eq(agent2.get_z3_id()).not());
                }
            }
        }

        optimizer.minimize(&timeline.times[timeline.times.len() - 1]);
        let _satisfied = optimizer.check(&[]);
        let model: Option<Model> = optimizer.get_model();

        let mut allocated_tasks: HashMap<String, AllocatedTask> = HashMap::new();
        match model {
            Some(sat_model) => {
                for (task_id, task) in &z3_tasks {
                    for (agent_id, agent) in &z3_agents {
                        if task.get_agent_code(&sat_model) == agent.get_agent_code(&sat_model) {
                            let task_times: (i64, i64) = task.get_final_times(&sat_model);
                            allocated_tasks.insert(
                                task_id.clone(),
                                AllocatedTask {
                                    id: task_id.clone(),
                                    primitives: task.task_info.primitives.clone(),
                                    agent_id: agent_id.clone(),
                                    start_time: task_times.0,
                                    end_time: task_times.1,
                                },
                            );
                        }
                    }
                }
                return Ok(allocated_tasks);
            }
            None => {
                return Err(optimizer.get_reason_unknown().unwrap_or("default".into()))
            }
        }

        ;
    }
}

pub struct PlanResult<'a> {
    pub satisfied: SatResult,
    pub model: Option<Model<'a>>,
    pub timeline: Timeline<'a>,
    pub z3_agents: HashMap<String, Z3Agent<'a>>,
    pub z3_tasks: HashMap<String, Z3Task<'a>>,
}
