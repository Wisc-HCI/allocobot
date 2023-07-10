use allocobot::description::job::Job;
use uuid::Uuid;
// use allocobot::petri::nets::basic::BasicNet;
// use allocobot::petri::nets::agent::AgentNet;
// use allocobot::petri::net::PetriNet;
// use plotly::common::color::Rgb;
// use plotly::common::{Fill, Line};
// use plotly::{Plot, Scatter};
// use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use serde_json;

fn main() -> std::io::Result<()> {
    // let colors: Vec<Rgb> = vec![
    //     Rgb::new(200, 100, 100),
    //     Rgb::new(100, 200, 100),
    //     Rgb::new(100, 100, 200),
    // ];

    let mut job = Job::new("Job 1".into());

    let _panda: Uuid =
        job.create_robot_agent("Panda".into(), 0.855, 3.0, 0.7, 2.0, 0.0001, 0.7, 2.0);
    let _charlie: Uuid = job.create_human_agent("Charlie".into());

    let _p1: Uuid = job.create_hand_point_of_interest("Point 1".into(), 0.0, 1.0, 0.1);
    let _p2: Uuid = job.create_hand_point_of_interest("Point 2".into(), 1.0, 1.0, 0.4);
    let _p3: Uuid = job.create_hand_point_of_interest("Point 3".into(), 0.5, 4.0, 0.4);

    let _p4: Uuid = job.create_standing_point_of_interest("Point 4".into(), 0.0, 0.0, 0.0);
    let _p5: Uuid = job.create_standing_point_of_interest("Point 5".into(), 1.0, 0.0, 0.0);

    let part1: Uuid = job.create_precursor_target("Part1".into(), 5.0, 5.0);
    let part2: Uuid = job.create_precursor_target("Part2".into(), 1.0, 3.0);
    let part3: Uuid = job.create_intermediate_target("Part3".into(), 6.0, 2.0);
    let part4: Uuid = job.create_precursor_target("Part4".into(), 14.0, 1.0);
    let part5: Uuid = job.create_product_target("Part5".into(), 4.0, 1.0);
    let part6: Uuid = job.create_product_target("Part6".into(), 10.0, 3.0);
    let tool0: Uuid = job.create_reusable_target("Tool0".into(), 1.0, 1.0);

    // let s1: Uuid = job.create_spawn_task("spawn1".into());
    // let s2: Uuid = job.create_spawn_task("spawn2".into());
    // let s3: Uuid = job.create_spawn_task("spawn3".into());

    // job.add_task_output(s1, part1, 1);
    // job.add_task_output(s2, part2, 1);
    // job.add_task_output(s3, part4, 1);

    let t1 = job.create_task("task1".into());
    let t2 = job.create_task("task2".into());

    job.add_task_dependency(t1,  part1, 1);
    job.add_task_dependency(t1,  part2, 1);
    job.add_task_output(t1, part3, 1);
    job.add_task_dependency(t2,  part3, 1);
    job.add_task_dependency(t2,  part4, 1);
    job.add_task_output(t2, part5, 1);
    job.add_task_output(t2, part6, 1);
    job.add_task_reusable(t1, tool0, 1);

    // let c1 = job.create_complete_task("complete1".into());
    // let c2 = job.create_complete_task("complete2".into());

    // job.add_task_dependency(c1, t2, part5);
    // job.add_task_dependency(c2, t2, part6);

    // let net_result = BasicNet::from_job(job);
    job.create_petri_nets();
    // let pois: Vec<PointOfInterest> = vec![p1, p2, p3];

    let mut jobfile = File::create("job.json")?;
    let mut basicfile = File::create("basic.dot")?;
    let mut agentfile = File::create("agent.dot")?;
    let mut poifile: File = File::create("poi.dot")?;
    let mut agent_net_file = File::create("agent_net.json")?;
    let mut poi_net_file = File::create("poi_net.json")?;

    jobfile.write_all(serde_json::to_string_pretty(&job).unwrap().as_bytes())?;

    basicfile.write_all(job.basic_net.unwrap().get_dot().as_bytes())?;

    // let agent_net = AgentNet::from((basic_net, agents));
    agentfile.write_all(job.agent_net.clone().unwrap().get_dot().as_bytes())?;

    agent_net_file.write_all(serde_json::to_string_pretty(&job.agent_net.clone().unwrap()).unwrap().as_bytes())?;

    poifile.write_all(job.poi_net.clone().unwrap().get_dot().as_bytes())?;

    poi_net_file.write_all(serde_json::to_string_pretty(&job.poi_net.clone().unwrap()).unwrap().as_bytes())?;

    Ok(())
}
