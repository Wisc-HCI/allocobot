use allocobot::description::agent::Agent;
// use allocobot::planner::Planner;
use allocobot::description::poi::PointOfInterest;
use allocobot::description::primitive::Primitive;
use allocobot::description::target::Target;
use allocobot::description::task::Task;
use allocobot::petri::net::BasicNet;
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

    let s1: Task = Task::new_spawn().with_name("s1".into()).with_output(&part1, 1);

    let t1: Task = Task::new_process()
        .with_name("task1".into())
        .with_primitive(Primitive::Selection {
            target: &part1,
            structure: 0.0,
            variability: 0.0,
            displacement: 0.0,
        })
        .with_primitive(Primitive::Grasp {
            target: &part1,
            structure: 0.0,
            variability: 0.0,
            displacement: 0.0,
            manipulation: 0.0,
            alignment: 0.0,
        })
        .with_dependency(&s1, &part1)
        .with_dependency(&s1, &part1)
        .with_output(&part1, 1);

    let t2: Task = Task::new_process()
        .with_name("task2".into())
        .with_primitive(Primitive::Release {
            target: &part1,
            structure: 0.0,
            variability: 0.0,
            manipulation: 0.0,
            alignment: 0.0,
        })
        .with_dependency(&t1, &part1)
        .with_output(&part1, 1)
        .with_poi(&pois["p1"])
        .with_poi(&pois["p2"]);

    let c1: Task = Task::new_complete()
        .with_name("c1".into())
        .with_dependency(&t2, &part1);

    println!("{:?}",t1.dependencies());

    let net_result = BasicNet::from_tasks("PRIME".into(), vec![&s1, &t1, &t2, &c1]);

    match net_result {
        Ok(net) => println!("{}", net),
        Err(e) => {
            eprintln!("{}", e)
        },
    }
}