use crate::description::job::Job;
use crate::description::primitive::Primitive;
use crate::petri::data::DataTag;
use crate::petri::transition::Transition;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, cmp, f64::consts::PI};
// use std::collections::HashMap;
use enum_tag::EnumTag;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Agent {
    Robot(RobotInfo),
    Human(HumanInfo),
}

impl Agent {
    pub fn new_robot(
        name: String,
        reach: f64,        // meters
        payload: f64,      // kg
        agility: f64,      // rating 0-1
        speed: f64,        // m/s
        precision: f64,    // m (repeatability)
        sensing: f64,      // rating 0-1
        mobile_speed: f64, // m/s
    ) -> Self {
        return Agent::Robot(RobotInfo {
            id: Uuid::new_v4(),
            name,
            reach,
            payload,
            agility,
            speed,
            precision,
            sensing,
            mobile_speed,
        });
    }

    pub fn new_human(
        name: String,
        assumption_age: f64, // Years
        assumption_acromial_height: f64, // meters
        assumption_reach: f64, // meters
        assumption_weight: f64, // kg
    ) -> Self {
        return Agent::Human(HumanInfo {
            id: Uuid::new_v4(),
            name,
            assumption_age,
            assumption_acromial_height,
            assumption_reach,
            assumption_weight,
        });
    }

    pub fn id(&self) -> Uuid {
        match self {
            Agent::Robot(robot_info) => return robot_info.id.clone(),
            Agent::Human(human_info) => return human_info.id.clone(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Agent::Robot(robot_info) => return robot_info.name.clone(),
            Agent::Human(human_info) => return human_info.name.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RobotInfo {
    pub id: Uuid,
    pub name: String,
    pub reach: f64,
    pub payload: f64,
    pub agility: f64,
    pub speed: f64,
    pub precision: f64,
    pub sensing: f64,
    pub mobile_speed: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HumanInfo {
    pub id: Uuid,
    pub name: String,
    // NOTE: This is not currently used, need to check with Rob
    // pub assumption_gender: Gender,
    pub assumption_age: f64,
    pub assumption_acromial_height: f64,
    pub assumption_reach: f64,
    pub assumption_weight: f64,
}

pub trait CostProfiler {
    fn execution_time(&self, transition: &Transition, job: &Job) -> usize;
    fn onetime_cost(&self, transition: &Transition, job: &Job) -> usize;
    fn ergo_cost_whole(&self, transition: &Transition, job: &Job) -> usize;
    fn ergo_cost_arm(&self, transition: &Transition, job: &Job) -> usize;
    fn ergo_cost_hand(&self, transition: &Transition, job: &Job) -> usize;
    fn ergo_recovery_whole(&self, transition: &Transition, job: &Job) -> usize;
    fn ergo_recovery_arm(&self, transition: &Transition, job: &Job) -> usize;
    fn ergo_recovery_hand(&self, transition: &Transition, job: &Job) -> usize;
}
// #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
// pub enum Gender {
//     Male,
//     Female,
//     // Other?
// }

impl CostProfiler for HumanInfo {
    fn execution_time(&self, transition: &Transition, job: &Job) -> usize {
        0
    }

    fn onetime_cost(&self, transition: &Transition, job: &Job) -> usize {
        let assigned_primitives: Vec<&Primitive> = transition
            .meta_data
            .iter()
            .filter(|d| d.tag() == DataTag::PrimitiveAssignment && d.id() == Some(self.id))
            .map(|d| job.primitives.get(&d.secondary().unwrap()).unwrap())
            .collect();

        let execution_time = self.execution_time(transition, job);

        let force_magnitude_on_target: HashMap<Uuid, f64> = assigned_primitives
            .iter()
            .filter_map(|p| match p {
                Primitive::Force {
                    magnitude, target, ..
                } => Some((*target, *magnitude)),
                _ => None,
            })
            .collect();

        let cost = match (assigned_primitives.len(), assigned_primitives.first()) {
            // if there is no assigned primitive
            (0, _) => {
                // This is a no-op, so just return 0
                0
            }
            (1, None) => {
                // Technically impossible, but we cover it anyway. Return 0
                0
            }
            (
                1,
                Some(Primitive::Carry {
                    target,
                    from_standing,
                    to_standing,
                    from_hand,
                    to_hand,
                    ..
                }),
            ) => {
                // Retrieve data
                let target_info = job.targets.get(target).unwrap();
                let from_standing_info = job.points_of_interest.get(from_standing).unwrap();
                let to_standing_info = job.points_of_interest.get(to_standing).unwrap();
                let from_hand_info = job.points_of_interest.get(from_hand).unwrap();
                let to_hand_info = job.points_of_interest.get(to_hand).unwrap();
                // Compute carry cost
                
                let starting_standing_vector = from_standing_info.position();
                let end_standing_vector = to_standing_info.position();

                let starting_hand_vector = from_hand_info.position();
                let end_hand_vector = to_hand_info.position();

                let acromial_vector: Vector3<f64> = Vector3::new(0.0, 0.0, self.assumption_acromial_height);
                let starting_acromial: Vector3<f64> = starting_standing_vector + acromial_vector;
                let end_acromial = end_standing_vector + acromial_vector;

                let walking_travel_vector:Vector3<f64> = to_standing_info.position() - from_standing_info.position();
                
                // Compute grade
                // 0% is flat, 100% is 90 degrees
                let grade = get_grade(starting_standing_vector, end_standing_vector);

                let walking_travel_distance = walking_travel_vector.norm();

                let comfortable_distance = 0.2 + target_info.size()/2.0;

                let starting_distance = (starting_hand_vector - starting_acromial).norm();
                let ending_distance = (end_hand_vector - end_acromial).norm();

                let starting_travel_distance = (starting_distance-comfortable_distance).abs();
                let end_travel_distance = (ending_distance-comfortable_distance).abs();

                let float_retrieve_cost = 0.01 * starting_travel_distance * ( 3.57 + 1.23 * target_info.weight() );
                
                let float_interim_cost = 0.01 * ( 68.0 + 0.23 * self.assumption_weight);

                let float_deposit_cost = 0.01 * end_travel_distance * ( 3.57 + 1.23 * target_info.weight() );
                
                let carry_cost = (float_retrieve_cost + float_interim_cost + float_deposit_cost) as usize;
                
                carry_cost
            }
            (
                1,
                Some(Primitive::Move {
                    target,
                    standing,
                    from_hand,
                    to_hand,
                    ..
                }),
            ) => {
                // Retrieve data
                let target_info = job.targets.get(target).unwrap();
                let standing_info = job.points_of_interest.get(standing).unwrap();
                let from_hand_info = job.points_of_interest.get(from_hand).unwrap();
                let to_hand_info = job.points_of_interest.get(to_hand).unwrap();
                // Compute carry cost
                0
            }
            (
                1,
                Some(Primitive::Travel {
                    from_standing,
                    to_standing,
                    from_hand,
                    to_hand,
                    ..
                }),
            ) => {
                // Retrieve data
                let from_standing_info = job.points_of_interest.get(from_standing).unwrap();
                let to_standing_info = job.points_of_interest.get(to_standing).unwrap();
                let from_hand_info = job.points_of_interest.get(from_hand).unwrap();
                let to_hand_info = job.points_of_interest.get(to_hand).unwrap();
                // Compute carry cost
                0
            }
            (
                1,
                Some(Primitive::Reach {
                    standing,
                    from_hand,
                    to_hand,
                    ..
                }),
            ) => {
                // Retrieve data
                let standing_info = job.points_of_interest.get(standing).unwrap();
                let from_hand_info = job.points_of_interest.get(from_hand).unwrap();
                let to_hand_info = job.points_of_interest.get(to_hand).unwrap();
                // Compute carry cost
                0
            }
            _ => {
                // There is some non-zero number of assigned primitives. Compute them independently and sum them
                let mut total_cost = 0;
                for assigned_primitive in assigned_primitives {
                    match assigned_primitive {
                        Primitive::Selection { target, .. } => {}
                        Primitive::Inspect { target, .. } => {}
                        Primitive::Hold { target, .. } => {}
                        Primitive::Position { target, .. } => {}
                        Primitive::Use { target, .. } => {}
                        _ => {
                            // Handled elsewhere
                        }
                    }
                }

                total_cost
            }
        };
        // if there is one assigned and it is a carry
        /*
            - Compute the cost of bringing the object to a comfortable holding position, based on the standing/hand pois
            - Compute the cost of moving the object while in a comfortable holding position from old standing to new standing
            - Compute the cost of bringing the object from a comfortable holding position to the new hand poi.
        */

        // if there is one assigned and it is a move
        /*
            - Compute the cost of moving the object directly from old hand to new hand
        */

        // if there is one assigned and it is a travel
        /*
            - Compute the cost of bringing the hands to a comfortable holding position, based on the standing/hand pois
            - Compute the cost of moving the hands while in a comfortable holding position from old standing to new standing
            - Compute the cost of bringing the hands from a comfortable holding position to the new hand poi.
        */

        // if there is one assigned and it is a reach
        /*
            - Compute the cost of moving the hands directly from old hand to new hand poi
        */

        cost
    }

    fn ergo_cost_whole(&self, transition: &Transition, job: &Job) -> usize {
        0
    }

    fn ergo_cost_arm(&self, transition: &Transition, job: &Job) -> usize {
        0
    }

    fn ergo_cost_hand(&self, transition: &Transition, job: &Job) -> usize {
        0
    }

    fn ergo_recovery_whole(&self, transition: &Transition, job: &Job) -> usize {
        0
    }

    fn ergo_recovery_arm(&self, transition: &Transition, job: &Job) -> usize {
        0
    }

    fn ergo_recovery_hand(&self, transition: &Transition, job: &Job) -> usize {
        0
    }
}

impl CostProfiler for RobotInfo {
    fn execution_time(&self, transition: &Transition, job: &Job) -> usize {
        0
    }

    fn onetime_cost(&self, transition: &Transition, primitives: &Job) -> usize {
        0
    }

    fn ergo_cost_whole(&self, transition: &Transition, job: &Job) -> usize {
        0
    }

    fn ergo_cost_arm(&self, transition: &Transition, job: &Job) -> usize {
        0
    }

    fn ergo_cost_hand(&self, transition: &Transition, job: &Job) -> usize {
        0
    }

    fn ergo_recovery_whole(&self, transition: &Transition, job: &Job) -> usize {
        0
    }

    fn ergo_recovery_arm(&self, transition: &Transition, job: &Job) -> usize {
        0
    }

    fn ergo_recovery_hand(&self, transition: &Transition, job: &Job) -> usize {
        0
    }
}

fn get_grade(point1: Vector3<f64>,point2: Vector3<f64>) -> f64 {
    let distance = (point1 - point2).norm();
    let height = point1.z - point2.z;
    if height.abs() > distance && distance != 0.0 {
        return ((PI/2.0 - (distance/height).asin())/(PI/2.0)).abs()
    } else if height.abs() < distance && distance != 0.0 {
        return ((height/distance).asin() / (PI/2.0)).abs()
    } else if height.abs() == distance && distance != 0.0 {
        return 1.0
    } else {
        return 0.0
    }
}

#[test]
fn test_grade() {
    let point1 = Vector3::new(0.0,0.0,0.0);
    let point2 = Vector3::new(1.0,1.0,0.0);
    assert_eq!(get_grade(point1,point2),0.0);

    let point3 = Vector3::new(0.0,0.0,0.0);
    assert_eq!(get_grade(point1, point3),0.0);

    let point4 = Vector3::new(1.0,0.0,1.0);
    assert!((get_grade(point1, point4) - 0.5) < f64::EPSILON);
    assert!((get_grade(point4, point1) - 0.5) < f64::EPSILON);

    let point5 = Vector3::new(1.0,1.0,1.0);
    assert_eq!(get_grade(point2,point5), 1.0);

    let point6 = Vector3::new(1.1,1.1,0.95);
    assert!(get_grade(point2,point6) > 0.9 && get_grade(point2,point6) < 1.0);

    let point7 = Vector3::new(0.0,0.0,1.1);
    assert_eq!(get_grade(point1,point7), 1.0);
}
