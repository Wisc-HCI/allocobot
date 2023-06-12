use allocobot::description::agent::Agent;
// use allocobot::planner::Planner;
use allocobot::description::poi::PointOfInterest;
// use allocobot::description::primitive::Primitive;
use allocobot::description::target::Target;
use allocobot::description::task::Task;
use allocobot::petri::nets::basic::BasicNet;
use allocobot::petri::nets::net::PetriNet;
// use plotly::common::color::Rgb;
// use plotly::common::{Fill, Line};
// use plotly::{Plot, Scatter};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    // let colors: Vec<Rgb> = vec![
    //     Rgb::new(200, 100, 100),
    //     Rgb::new(100, 200, 100),
    //     Rgb::new(100, 100, 200),
    // ];

    // define Human and Robot Agents
    let _agents: HashMap<String, Agent> = HashMap::from([
        ("charlie".into(), Agent::new_human("charlie".into())),
        (
            "panda".into(),
            Agent::new_robot("Panda".into(), 0.855, 3.0, 0.7, 2.0, 0.0001, 0.7, false),
        ),
    ]);

    // let parts = vec![];

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

    let part1: Target = Target::new("Part1".into(), 5.0, 5.0);
    let part2: Target = Target::new("Part2".into(), 1.0, 3.0);
    let part3: Target = Target::new("Part3".into(), 6.0, 2.0);
    let part4: Target = Target::new("Part4".into(), 14.0, 1.0);
    let part5: Target = Target::new("Part5".into(), 4.0, 1.0);
    let part6: Target = Target::new("Part6".into(), 10.0, 3.0);

    let s1: Task = Task::new_spawn()
        .with_name("s1".into())
        .with_output(&part1, 1);
    let s2: Task = Task::new_spawn()
        .with_name("s2".into())
        .with_output(&part2, 1);
    let s3: Task = Task::new_spawn()
        .with_name("s3".into())
        .with_output(&part4, 1);

    let t1: Task = Task::new_process()
        .with_name("task1".into())
        // .with_primitive(Primitive::Selection {
        //     target: &part1,
        //     structure: 0.0,
        //     variability: 0.0,
        //     displacement: 0.0,
        // })
        // .with_primitive(Primitive::Grasp {
        //     target: &part1,
        //     structure: 0.0,
        //     variability: 0.0,
        //     displacement: 0.0,
        //     manipulation: 0.0,
        //     alignment: 0.0,
        // })
        .with_dependency(&s1, &part1)
        .with_dependency(&s2, &part2)
        .with_output(&part3, 1);

    let t2: Task = Task::new_process()
        .with_name("task2".into())
        // .with_primitive(Primitive::Release {
        //     target: &part1,
        //     structure: 0.0,
        //     variability: 0.0,
        //     manipulation: 0.0,
        //     alignment: 0.0,
        // })
        .with_dependency(&t1, &part3)
        .with_dependency(&s3, &part4)
        .with_output(&part5, 1)
        .with_output(&part6, 1)
        .with_poi(&pois["p1"])
        .with_poi(&pois["p2"]);

    let c1: Task = Task::new_complete()
        .with_name("c1".into())
        .with_dependency(&t2, &part5);

    let c2: Task = Task::new_complete()
        .with_name("c2".into())
        .with_dependency(&t2, &part6);

    let net_result = BasicNet::from_tasks(
        "PRIME".into(), 
        vec![
            &s1, 
            &s2, 
            &s3, 
            &t1, 
            &t2, 
            &c1, 
            &c2
        ]
    );

    match net_result {
        Ok(net) => {
            println!("{:#?}", net);
            let mut file = File::create("basic.dot")?;
            file.write_all(net.get_dot().as_bytes())?;
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", e);
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Error"))
        }
    }
}
