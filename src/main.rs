use plotly::common::color::Rgb;
use plotly::common::{Fill, Line};
use plotly::{Plot, Scatter};
use std::collections::HashMap;
use z3::ast;
use z3::ast::Ast;
use z3::Model;
use z3::{Config, Context, Optimize};

fn main() {
    let colors: Vec<Rgb> = vec![
        Rgb::new(200, 100, 100),
        Rgb::new(100, 200, 100),
        Rgb::new(100, 100, 200),
    ];
    let ctx: &Context = &Context::new(&Config::default());
    let optimizer: Optimize = Optimize::new(ctx);
    // define Human
    let charlie: Agent = Agent::new_human(ctx, "001".into(), "charlie".into());
    // define Robot
    let panda: Agent = Agent::new_robot(ctx, "002".into(), "panda".into());

    let agents: Vec<&Agent> = vec![&charlie, &panda];

    let tasks: Vec<Task> = vec![
        Task::new(ctx, "Task1".into(), Some(&charlie), 5000),
        Task::new(ctx, "Task2".into(), Some(&panda), 3000),
        Task::new(ctx, "Task3".into(), None, 3000),
    ];

    let timeline: Timeline = Timeline::new(ctx, tasks.iter().collect(), &agents);

    for assertion in &timeline.assertions {
        optimizer.assert(assertion)
    }

    for task in tasks.iter() {
        for assertion in task.get_assertions(ctx, &timeline, &agents).iter() {
            optimizer.assert(assertion)
        }
    }

    // Require that the first event is zero
    optimizer.assert(&timeline.times[0]._eq(&ast::Int::from_i64(ctx, 0)));

    // Each time in the timeline must be greater than the one before it
    for event_idx in 0..(timeline.times.len() - 1) {
        optimizer.assert(&timeline.times[event_idx].lt(&timeline.times[event_idx + 1]))
    }

    // Require that the agent markers are distinct
    for (i, agent1) in agents.iter().enumerate() {
        for (j, agent2) in agents.iter().enumerate() {
            if i < j {
                optimizer.assert(&agent1.get_z3_id()._eq(agent2.get_z3_id()).not());
            }
        }
    }

    optimizer.minimize(&timeline.times[timeline.times.len() - 1]);

    let satisfied = optimizer.check(&[]);

    let model: Option<Model> = optimizer.get_model();

    println!(
        "{:?}\n {:?}\n {:?}\n",
        satisfied,
        model,
        optimizer.get_reason_unknown()
    );

    match model {
        Some(sat_model) => {
            let mut plot = Plot::new();
            for event in timeline.times {
                println!("{:?}", sat_model.eval(&event, true))
            }
            for (agent_idx, agent) in agents.iter().enumerate() {
                for task in &tasks {
                    if task.get_agent_code(&sat_model) == agent.get_agent_code(&sat_model) {
                        let task_times: (i64, i64) = task.get_final_times(&sat_model);
                        let trace = Scatter::new(
                            vec![task_times.0, task_times.1, task_times.1, task_times.0],
                            vec![agent_idx, agent_idx, agent_idx + 1, agent_idx + 1],
                        )
                        .line(Line::new().color(colors[agent_idx]))
                        .fill(Fill::ToSelf)
                        .name(format!("{} - {}", agent.get_name(), task.id).as_str());
                        // .legend_group_title(LegendGroupTitle::new(format!("{} - {}",agent.get_id(),task.id).as_str()));
                        plot.add_trace(trace)
                    }
                }
            }
            plot.show();
        }
        None => {
            println!("No solutions")
        }
    }
}

pub enum Agent<'a> {
    Robot(AgentInfo<'a>),
    Human(AgentInfo<'a>),
}

impl<'a> Agent<'a> {
    pub fn new_robot(ctx: &'a Context, id: String, name: String) -> Self {
        return Agent::Robot(AgentInfo {
            id: id.clone(),
            name,
            z3_id: ast::Int::new_const(ctx, id.clone()),
        });
    }

    pub fn new_human(ctx: &'a Context, id: String, name: String) -> Self {
        return Agent::Human(AgentInfo {
            id: id.clone(),
            name,
            z3_id: ast::Int::new_const(ctx, id.clone()),
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

    pub fn get_z3_id(&self) -> &ast::Int {
        match self {
            Agent::Robot(robot_info) => return &robot_info.z3_id,
            Agent::Human(human_info) => return &human_info.z3_id,
        }
    }

    pub fn get_agent_code(&self, model: &Model) -> i64 {
        return model
            .eval(self.get_z3_id(), true)
            .map_or(0, |z3inner| z3inner.as_i64().unwrap_or(0));
    }
}

pub struct Task<'a> {
    pub id: String,
    pub agent: Option<&'a Agent<'a>>,
    pub z3_agent: ast::Int<'a>,
    pub z3_start_time: ast::Int<'a>,
    pub z3_end_time: ast::Int<'a>,
    pub z3_duration: ast::Int<'a>,
}

impl<'a> Task<'a> {
    pub fn new(ctx: &'a Context, id: String, agent: Option<&'a Agent>, duration: i64) -> Self {
        return Self {
            id: id.clone(),
            agent,
            z3_agent: ast::Int::new_const(ctx, format!("agent-{}", id.clone())),
            z3_start_time: ast::Int::new_const(ctx, format!("start-{}", id.clone())),
            z3_end_time: ast::Int::new_const(ctx, format!("end-{}", id.clone())),
            z3_duration: ast::Int::from_i64(ctx, duration),
        };
    }

    pub fn get_assertions(
        &'a self,
        ctx: &'a Context,
        timeline: &Timeline<'a>,
        agents: &Vec<&'a Agent<'a>>,
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
        for agent in agents {
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

        // For each agent, if assigned, then the robot is busy between start and end
        for agent in agents {
            let agent_is_assigned: ast::Bool = self.z3_agent._eq(&agent.get_z3_id());
            let agent_task_busy_rules = timeline.times
                .iter().enumerate().map(
                    |(time_idx,time)| ast::Bool::and(ctx, &[
                        &time.ge(&self.z3_start_time),
                        &time.le(&self.z3_end_time)
                    ]).implies(
                        &timeline.busy_grid
                            .get(&agent.get_id())
                            .unwrap()
                            .get(&self.id)
                            .unwrap()[time_idx]
                    )      
                )
                .collect::<Vec<ast::Bool>>();
            assertions.push(agent_is_assigned.implies(
                &ast::Bool::and(ctx,&agent_task_busy_rules.iter().collect::<Vec<&ast::Bool>>().as_slice())
            ));
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

pub struct AgentInfo<'a> {
    pub id: String,
    pub name: String,
    pub z3_id: ast::Int<'a>,
}

pub struct Timeline<'a> {
    pub times: Vec<ast::Int<'a>>,
    pub busy_grid: HashMap<String, HashMap<String, Vec<ast::Bool<'a>>>>,
    pub assertions: Vec<ast::Bool<'a>>,
}

impl<'a> Timeline<'a> {
    pub fn new(ctx: &'a Context, tasks: Vec<&Task>, agents: &Vec<&Agent>) -> Self {
        let times: Vec<ast::Int> = (0..tasks.len() * 3)
            .map(|i| ast::Int::new_const(ctx, format!("t{}", i)))
            .collect();
        let mut busy_grid: HashMap<String, HashMap<String, Vec<ast::Bool<'a>>>> = HashMap::new();

        for agent in agents {
            let mut agent_busy_grid: HashMap<String, Vec<ast::Bool<'a>>> = HashMap::new();
            for task in &tasks {
                agent_busy_grid.insert(
                    task.id.clone(),
                    (0..tasks.len() * 3)
                        .map(|i| {
                            ast::Bool::new_const(
                                ctx,
                                format!("b_{}_{}_{}", agent.get_id(), task.id, i),
                            )
                        })
                        .collect(),
                );
            }
            busy_grid.insert(agent.get_id(), agent_busy_grid);
        }

        let mut assertions: Vec<ast::Bool> = Vec::new();

        for time_idx in 0..times.len() {
            for agent in agents {
                let mut agent_does_task: Vec<&ast::Bool> = Vec::new();
                for task in &tasks {
                    agent_does_task.push(
                        &busy_grid
                            .get(&agent.get_id())
                            .unwrap()
                            .get(&task.id)
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
