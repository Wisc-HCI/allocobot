use allocobot::agents::Agent;
use allocobot::planner::Planner;
use allocobot::tasks::{TaskInfo};
use plotly::common::color::Rgb;
use plotly::common::{Fill, Line};
use plotly::{Plot, Scatter};
use std::collections::HashMap;

fn main() {
    let colors: Vec<Rgb> = vec![
        Rgb::new(200, 100, 100),
        Rgb::new(100, 200, 100),
        Rgb::new(100, 100, 200),
    ];

    

    // define Human
    let agents: HashMap<String,Agent> = HashMap::from([
        ("charlie".into(),Agent::new_human("charlie".into(), "Charlie".into())),
        ("panda".into(),Agent::new_robot("panda".into(), "Panda".into()))
    ]);

    let tasks: HashMap<String,TaskInfo> = HashMap::from([
        ("task1".into(),TaskInfo::new("task1".into(), vec![],Some("charlie".into()), 5000, vec![])),
        ("task2".into(),TaskInfo::new("task2".into(), vec![],Some("panda".into()), 3000, vec![])),
        ("task3".into(),TaskInfo::new("task3".into(), vec![],None, 3000, vec!["task1".into()])),
        ("task4".into(),TaskInfo::new("task4".into(), vec![],None, 2500, vec![])),
        ("task5".into(),TaskInfo::new("task5".into(), vec![],None, 2000, vec![])),
        ("task6".into(),TaskInfo::new("task6".into(), vec![],None, 1000, vec!["task3".into()]))
    ]);

    let mut planner = Planner::new(&tasks,&agents);

    let result = planner.plan();
    let mut agent_ids: Vec<String> = vec![];
    let mut agent_names: Vec<String> = vec![];
    for agent in agents.values() {
        agent_ids.push(agent.get_id());
        agent_names.push(agent.get_name());
    }
    println!("{:?}",agent_ids);

    match result {
        Ok(allocated_tasks) => {
            let mut plot = Plot::new();
            for (task_id,  task) in &allocated_tasks {
                println!("{:?}",task);
                let agent_idx = agent_ids.iter().position(|v| v==&task.agent_id).unwrap_or(0);
                let agent_name = agents.get(&task.agent_id).unwrap().get_name();
                
                let trace = Scatter::new(
                    vec![task.start_time, task.end_time, task.end_time, task.start_time],
                    vec![agent_idx, agent_idx, agent_idx + 1, agent_idx + 1],
                )
                .line(Line::new().color(colors[agent_idx]))
                .fill(Fill::ToSelf)
                .name(format!("{} - {}", agent_name, task_id).as_str());
                // .legend_group_title(LegendGroupTitle::new(format!("{} - {}",agent.get_id(),task.id).as_str()));
                plot.add_trace(trace);
            }
            plot.show();
        }
        Err(msg) => {
            println!("{}",msg)
        }
    }
}
