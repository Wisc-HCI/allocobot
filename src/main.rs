use allocobot::agents::Agent;
use allocobot::planner::Planner;
use allocobot::poi::PointOfInterest;
use allocobot::primitives::Primitive;
use allocobot::tasks::TaskInfo;
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

    // define Human and Robot Agents
    let agents: HashMap<String, Agent> = HashMap::from([
        (
            "charlie".into(),
            Agent::new_human("charlie".into(), "Charlie".into()),
        ),
        (
            "panda".into(),
            Agent::new_robot(
                "panda".into(),
                "Panda".into(),
                0.855,
                3.0,
                0.7,
                2.0,
                0.0001,
                0.7,
                false,
            ),
        ),
    ]);

    let pois: HashMap<String, PointOfInterest> = HashMap::from([
        (
            "p1".into(),
            PointOfInterest::new("p1".into(), "Point 1".into(), 0.0, 1.0, 0.1),
        ),
        (
            "p2".into(),
            PointOfInterest::new("p2".into(), "Point 2".into(), 1.0, 1.1, 0.4),
        ),
        (
            "p3".into(),
            PointOfInterest::new("p3".into(), "Point 3".into(), 0.5, 4.0, 0.4),
        ),
    ]);

    let tasks: HashMap<String, TaskInfo> = HashMap::from([
        (
            "get_and_take_protector".into(),
            TaskInfo::new("get_and_take_protector".into(), vec![

            ], None, 5000, vec![]),
        ),
        (
            "insert_blue_protector".into(),
            TaskInfo::new(
                "insert_blue_protector".into(),
                vec![
                    
                ],
                None,
                3000,
                vec!["get_and_take_protector".into()],
            ),
        ),
        (
            "get_half_shaft".into(),
            TaskInfo::new("get_half_shaft".into(), vec![

            ], None, 3000, vec![]),
        ),
        (
            "install_half_shaft".into(),
            TaskInfo::new("install_half_shaft".into(), vec![

            ], None, 2500, vec![]),
        ),
        (
            "dispose_blue_protector".into(),
            TaskInfo::new("dispose_blue_protector".into(), vec![

            ], None, 2000, vec![]),
        ),
        (
            "get_stab_bolt1".into(),
            TaskInfo::new("get_stab_bolt1".into(), vec![

            ], None, 1000, vec![]),
        ),
        (
            "get_stab_bolt2".into(),
            TaskInfo::new("get_stab_bolt2".into(), vec![
                
            ], None, 1000, vec![]),
        ),
        // (
        //     "install_harness".into(),
        //     TaskInfo::new("install_harness".into(), vec![], None, 1000, vec![]),
        // ),
        // (
        //     "place_front_strut".into(),
        //     TaskInfo::new("place_front_strut".into(), vec![], None, 1000, vec![]),
        // ),
        // (
        //     "scan_half_shaft".into(),
        //     TaskInfo::new("scan_half_shaft".into(), vec![], None, 1000, vec![]),
        // ),
        // (
        //     "place_x_tool".into(),
        //     TaskInfo::new("place_x_tool".into(), vec![], None, 1000, vec![]),
        // ),
        // (
        //     "manual_retention_check".into(),
        //     TaskInfo::new("manual_retention_check".into(), vec![], None, 1000, vec![]),
        // ),
        // (
        //     "x_tool_retention_check".into(),
        //     TaskInfo::new("x_tool_retention_check".into(), vec![], None, 1000, vec![]),
        // ),
        // (
        //     "remove_x_tool".into(),
        //     TaskInfo::new("remove_x_tool".into(), vec![], None, 1000, vec![]),
        // ),
    ]);

    let mut planner = Planner::new(&tasks, &agents, &pois);

    let result = planner.plan();
    let mut agent_ids: Vec<String> = vec![];
    let mut agent_names: Vec<String> = vec![];
    for agent in agents.values() {
        agent_ids.push(agent.get_id());
        agent_names.push(agent.get_name());
    }
    println!("{:?}", agent_ids);

    match result {
        Ok(allocated_tasks) => {
            let mut plot = Plot::new();
            for (task_id, task) in &allocated_tasks {
                println!("{:?}", task);
                let agent_idx = agent_ids
                    .iter()
                    .position(|v| v == &task.agent_id)
                    .unwrap_or(0);
                let agent_name = agents.get(&task.agent_id).unwrap().get_name();

                let trace = Scatter::new(
                    vec![
                        task.start_time,
                        task.end_time,
                        task.end_time,
                        task.start_time,
                    ],
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
            println!("{}", msg)
        }
    }
}
