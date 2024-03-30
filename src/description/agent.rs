use crate::constants::{DISTANCE_PER_PACE, TMU_PER_SECOND};
use crate::description::job::Job;
use crate::description::primitive::Primitive;
use crate::description::rating::Rating;
use crate::description::poi::PointOfInterest;
use crate::description::units::Time;
use crate::petri::data::{Data, DataTag};
use crate::petri::transition::Transition;
use crate::petri::cost::{CostSet, CostFrequency, CostCategory, Cost};
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use std::{cmp, collections::HashMap, f64::consts::PI};
// use std::collections::HashMap;
use enum_tag::EnumTag;
use uuid::Uuid;

use super::target;
use super::units::TokenCount;



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
        agility: Rating,      // rating 0-1
        speed: f64,        // m/s
        precision: f64,    // m (repeatability)
        sensing: Rating,      // rating 0-1
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
        age: f64,             // Years
        acromial_height: f64, // meters
        height: f64, // meters
        reach: f64,           // meters
        weight: f64,          // kg
        skill: Rating,
    ) -> Self {
        return Agent::Human(HumanInfo {
            id: Uuid::new_v4(),
            name,
            age,
            acromial_height,
            height,
            reach,
            weight,
            skill
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
    pub agility: Rating,
    pub speed: f64,
    pub precision: f64,
    pub sensing: Rating,
    pub mobile_speed: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HumanInfo {
    pub id: Uuid,
    pub name: String,
    // NOTE: This is not currently used, need to check with Rob
    // pub gender: Gender,
    pub age: f64,
    pub acromial_height: f64,
    pub height: f64,
    pub reach: f64,
    pub weight: f64,
    pub skill: Rating
}

pub trait CostProfiler {
    fn execution_time(&self, transition: &Transition, job: &Job) -> Time;
    fn cost_set(&self, transition: &Transition, job: &Job) -> CostSet;
    fn ergo_cost_whole(&self, transition: &Transition, job: &Job) -> TokenCount;
    fn ergo_cost_arm(&self, transition: &Transition, job: &Job) -> TokenCount;
    fn ergo_cost_hand(&self, transition: &Transition, job: &Job) -> TokenCount;
    fn ergo_recovery_whole(&self, transition: &Transition, job: &Job) -> TokenCount;
    fn ergo_recovery_arm(&self, transition: &Transition, job: &Job) -> TokenCount;
    fn ergo_recovery_hand(&self, transition: &Transition, job: &Job) -> TokenCount;
}

impl CostProfiler for HumanInfo {
    fn execution_time(&self, transition: &Transition, job: &Job) -> Time {
        let assigned_primitives = get_assigned_primitives(transition, job, self.id);
        
        let mut max_time = 0.0;

        for primitive in assigned_primitives.iter() {
            let temp_vec = vec![*primitive];
            let single_time = get_human_time_for_primitive(temp_vec, job, self);
            if single_time > max_time {
                max_time = single_time;
            }

            for primitive_two in assigned_primitives.iter() {
                let temp_vec = vec![*primitive, *primitive_two];
                let doubles_time = get_human_time_for_primitive(temp_vec, job, self);
                if doubles_time > max_time {
                    max_time = doubles_time;
                }
            }
        }

        return max_time;
    }

    fn cost_set(&self, transition: &Transition, job: &Job) -> CostSet {
        let assigned_primitives: Vec<&Primitive> = transition
            .meta_data
            .iter()
            .filter(|d| d.tag() == DataTag::PrimitiveAssignment && d.id() == Some(self.id))
            .map(|d| job.primitives.get(&d.secondary().unwrap()).unwrap())
            .collect();

        let mut one_time_ergo_cost = CostSet::new();

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
                0.0
            }
            (1, None) => {
                // Technically impossible, but we cover it anyway. Return 0
                0.0
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

                let acromial_vector: Vector3<f64> =
                    Vector3::new(0.0, 0.0, self.acromial_height);
                let starting_acromial: Vector3<f64> = starting_standing_vector + acromial_vector;
                let end_acromial = end_standing_vector + acromial_vector;

                let walking_travel_vector: Vector3<f64> =
                    to_standing_info.position() - from_standing_info.position();

                // Compute grade
                // 0% is flat, 100% is 90 degrees
                let grade = get_grade(starting_standing_vector, end_standing_vector);

                let walking_travel_distance = walking_travel_vector.norm();

                let comfortable_distance = 0.2 + target_info.size() / 2.0;

                let starting_distance = (starting_hand_vector - starting_acromial).norm();
                let ending_distance = (end_hand_vector - end_acromial).norm();

                let starting_travel_distance = (starting_distance - comfortable_distance).abs();
                let end_travel_distance = (ending_distance - comfortable_distance).abs();

                let float_retrieve_cost =
                    0.01 * starting_travel_distance * (3.57 + 1.23 * target_info.weight());

                let float_interim_cost = 0.01 * (68.0 + 0.23 * self.weight);

                let float_deposit_cost =
                    0.01 * end_travel_distance * (3.57 + 1.23 * target_info.weight());

                let carry_cost =
                    float_retrieve_cost + float_interim_cost + float_deposit_cost;

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
                0.0
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
                0.0
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
                0.0
            }
            _ => {
                // There is some non-zero number of assigned primitives. Compute them independently and sum them
                let mut total_cost = 0.0;
                for assigned_primitive in assigned_primitives {
                    match assigned_primitive {
                        Primitive::Hold { target, .. } => {
                            let force_on_hold_target =
                                force_magnitude_on_target.get(target).unwrap_or(&0.0);
                            let mut hand_poi: &PointOfInterest;
                            let mut standing_poi: &PointOfInterest;
                            for meta_datum in transition.meta_data.iter() {
                                match meta_datum {
                                    Data::Hand(poi_id, agent_id) => {
                                        hand_poi = job.points_of_interest.get(&poi_id).unwrap();
                                    }
                                    Data::Standing(poi_id, agent_id) => {
                                        standing_poi = job.points_of_interest.get(&poi_id).unwrap();
                                    }
                                    _ => {
                                        // Handled elsewhere
                                    }
                                }
                            }
                            // let target_poi = job.points_of_interest

                            if force_on_hold_target == &0.0 {
                                // // A static hold.
                                // /*
                                // Incremental Energy Function:
                                // dE = .01(
                                //         80 +
                                //         2.43*body_weight*(walking_speed)^2 +
                                //         4.63*(target_weight)*(walking_speed)^2 +
                                //         4.62*(target_weight)
                                //     )*(duration) +
                                //     .379(
                                //         (target_weight) +
                                //         body_weight
                                //     )*(grade)*(walking_speed)*(duration)
                                //  */
                                // let target_info = job.targets.get(target).unwrap();

                                // let incremental_energy = 0.01 * (
                                //     80.0 +
                                //     2.43 * self.weight * 1.0 +//self.mobile_speed.powi(2) +
                                //     4.63 * target_info.weight() * 1.0 +//self.mobile_speed.powi(2) +
                                //     4.62 * target_info.weight()
                                // ) * execution_time as f64;
                                // // The remainder cancels out because there is no grade
                                // println!("Incremental Energy: {}", incremental_energy);
                                // total_cost += incremental_energy as usize;
                            } else {
                                // Compute the cost of holding the object while applying force
                            }
                        }
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

        // cost
        one_time_ergo_cost.push(Cost {
            frequency: CostFrequency::Once,
            value: cost,
            category: CostCategory::Ergonomic
        });
        one_time_ergo_cost
    }

    fn ergo_cost_whole(&self, transition: &Transition, job: &Job) -> TokenCount {
        0
    }

    fn ergo_cost_arm(&self, transition: &Transition, job: &Job) -> TokenCount {
        0
    }

    fn ergo_cost_hand(&self, transition: &Transition, job: &Job) -> TokenCount {
        0
    }

    fn ergo_recovery_whole(&self, transition: &Transition, job: &Job) -> TokenCount {
        0
    }

    fn ergo_recovery_arm(&self, transition: &Transition, job: &Job) -> TokenCount {
        0
    }

    fn ergo_recovery_hand(&self, transition: &Transition, job: &Job) -> TokenCount {
        0
    }
}

impl CostProfiler for RobotInfo {
    fn execution_time(&self, transition: &Transition, job: &Job) -> Time {
        0.0
    }

    fn cost_set(&self, transition: &Transition, job: &Job) -> CostSet {
        vec![]
    }

    fn ergo_cost_whole(&self, transition: &Transition, job: &Job) -> TokenCount {
        0
    }

    fn ergo_cost_arm(&self, transition: &Transition, job: &Job) -> TokenCount {
        0
    }

    fn ergo_cost_hand(&self, transition: &Transition, job: &Job) -> TokenCount {
        0
    }

    fn ergo_recovery_whole(&self, transition: &Transition, job: &Job) -> TokenCount {
        0
    }

    fn ergo_recovery_arm(&self, transition: &Transition, job: &Job) -> TokenCount {
        0
    }

    fn ergo_recovery_hand(&self, transition: &Transition, job: &Job) -> TokenCount {
        0
    }
}

fn get_grade(point1: Vector3<f64>, point2: Vector3<f64>) -> f64 {
    let distance = (point1 - point2).norm();
    let height = point1.z - point2.z;
    if height.abs() > distance && distance != 0.0 {
        return ((PI / 2.0 - (distance / height).asin()) / (PI / 2.0)).abs();
    } else if height.abs() < distance && distance != 0.0 {
        return ((height / distance).asin() / (PI / 2.0)).abs();
    } else if height.abs() == distance && distance != 0.0 {
        return 1.0;
    } else {
        return 0.0;
    }
}

const HUMAN_WEIGHT_FACTORS_MAPPING: [(f64, f64, f64, f64); 9] = [
    // (LOWER BOUND, UPPER BOUND, FACTOR, CONSTANT)
    (1.13, 3.4, 1.06, 2.2),
    (3.4, 5.67, 1.11, 3.9),
    (5.67, 7.94, 1.17, 5.6),
    (7.94, 10.21, 1.22, 7.4),
    (10.21, 12.5, 1.28, 9.1),
    (12.5, 14.74, 1.33, 10.8),
    (14.74, 17.0, 1.39, 12.5),
    (17.0, 19.29, 1.44, 14.3),
    (19.29, f64::INFINITY, 1.5, 16.0)
];

fn get_assigned_primitives<'t>(transition: &'t Transition, job: &'t Job, agent_id: Uuid) -> Vec<&'t Primitive> {
    transition
        .meta_data
        .iter()
        .filter(|d| d.tag() == DataTag::PrimitiveAssignment && d.id() == Some(agent_id))
        .map(|d| job.primitives.get(&d.secondary().unwrap()).unwrap())
        .collect()
}

fn get_is_within_neutral_reach(standing_poi: &PointOfInterest, hand_poi: &PointOfInterest, acromial_height: f64, reach: f64) -> bool {
    let standing_vector = standing_poi.position();
    let hand_vector = hand_poi.position();
    let acromial_vector = Vector3::new(0.0, 0.0, acromial_height);
    let acromial_standing_vector = standing_vector + acromial_vector;
    let distance = (hand_vector - acromial_standing_vector).norm();
    return distance <= reach;
}

fn get_human_time_for_primitive(assigned_primitives: Vec<&Primitive>, job: &Job, agent: &HumanInfo) -> Time {
    return match (assigned_primitives.len(), assigned_primitives.first()) {
        (0, _) => {
            // This is a no-op, so just return 0
            0.0
        },
        (1, None) => {
            // Technically impossible, but we cover it anyway. Return 0
            0.0
        },
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
            // Consider Carry Calculation

            let mut tmu: f64 = 0.0;

            // Retrieve data
            let from_standing_info = job.points_of_interest.get(from_standing).unwrap();
            let to_standing_info = job.points_of_interest.get(to_standing).unwrap();
            let to_hand_info = job.points_of_interest.get(to_hand).unwrap();
            let from_hand_info = job.points_of_interest.get(from_hand).unwrap();

            // Grasp Time in TMU
            tmu += 2.0;

            // If the source hand is below the reachable area, based on standing location, add a time penalty
            if get_is_within_neutral_reach(from_standing_info, from_hand_info, agent.acromial_height, agent.reach) {
                tmu += 30.5;
            }

            
            // Compute travel vector/distance
            let travel_vector = to_standing_info.position() - from_standing_info.position();
            let travel_distance = travel_vector.norm();
            tmu += 17.0 * (travel_distance/1.19);


            // If the target hand is below the reachable area, based on standing location, add a tmu penalty
            if get_is_within_neutral_reach(to_standing_info, to_hand_info, agent.acromial_height, agent.reach) {
                tmu += 30.5;
            }

            // Release time
            tmu += 2.0;

            // Convert from TMU to seconds
            let time = tmu * TMU_PER_SECOND;

            return time;
        }, 
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
            // Consider Move Calculation

            let mut tmu: f64 = 0.0;

            // Retrieve data
            let standing_info = job.points_of_interest.get(standing).unwrap();
            let to_hand_info = job.points_of_interest.get(to_hand).unwrap();
            let from_hand_info = job.points_of_interest.get(from_hand).unwrap();
            let target_info = job.targets.get(target).unwrap();

            // Compute travel vector/distance
            let motion_vector = to_hand_info.position() - from_hand_info.position();
            let motion_distance = motion_vector.norm();

            // Calculate Grasp Time
            if target_info.size() > 0.0254 {
                tmu += 0.5;
            } else if 0.00635 <= target_info.size() && target_info.size() <= 0.0254 {
                tmu += 9.1;
            } else {
                tmu += 12.9;
            }

            // If the source hand is below the reachable area, based on standing location, add a tmu penalty
            if get_is_within_neutral_reach(standing_info, from_hand_info, agent.acromial_height, agent.reach) {
                tmu += 30.5;
            }

            // Movement tmu
            let mut movement_tmu = 0.0;
            match to_hand_info.variability() {
                Rating::High => {
                    if motion_distance < 0.0254 {
                        movement_tmu += 2.0
                    } else if 0.0254 <= motion_distance && motion_distance <= 0.1016 {
                        movement_tmu += 3.6866*motion_distance.powf(0.6146)
                    } else if 0.1016 <= motion_distance && motion_distance <= 0.762 {
                        movement_tmu += 5.959169 + 0.690797*motion_distance
                    } else {
                        movement_tmu += 5.959169 + 0.690797*motion_distance + 0.7*(motion_distance - 0.762)
                    }
                },
                Rating::Medium => {
                    if motion_distance < 0.0254 {
                        movement_tmu += 2.0
                    } else if 0.0254 <= motion_distance && motion_distance <= 0.1016 {
                        movement_tmu += 2.5*motion_distance.powf(0.681)
                    } else if 0.1016 <= motion_distance && motion_distance <= 0.762 {
                        movement_tmu += 4.309488 + 0.71666*motion_distance
                    } else {
                        movement_tmu += 4.309488 + 0.71666*motion_distance + 0.7*(motion_distance - 0.762)
                    }
                },
                Rating::Low => {
                    if motion_distance < 0.0254 {
                        movement_tmu += 2.0
                    } else if 0.0254 <= motion_distance && motion_distance <= 0.0762 {
                        movement_tmu += 2.5*motion_distance.powf(0.681)
                    } else if 0.0762 <= motion_distance && motion_distance <= 0.762 {
                        movement_tmu += 4.333601 + 0.440266*motion_distance
                    } else {
                        movement_tmu += 4.333601 + 0.440266*motion_distance + 0.4*(motion_distance - 0.762)
                    }
                }
            }

            // Depending on the weight of the object, add a tmu penalty
            let weight = target_info.weight();
            for (lower, upper, factor, constant) in HUMAN_WEIGHT_FACTORS_MAPPING {
                if lower < weight && weight <= upper {
                    movement_tmu += factor*weight + constant;
                    break;
                }
            }
            tmu += movement_tmu;

            if get_is_within_neutral_reach(standing_info, to_hand_info, agent.acromial_height, agent.reach) {
                tmu += 30.5;
            }
            
            // Release tmu
            tmu += 2.0;

            let time = tmu * TMU_PER_SECOND;

            return time;
        },
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
            // Consider Travel Calculation

            // Retrieve data
            let from_standing_info = job.points_of_interest.get(from_standing).unwrap();
            let to_standing_info = job.points_of_interest.get(to_standing).unwrap();

            // Compute travel vector/distance
            let travel_vector = to_standing_info.position() - from_standing_info.position();
            let travel_distance = travel_vector.norm();

            // Compute carry time
            return (15.0 * (travel_distance / DISTANCE_PER_PACE)) * TMU_PER_SECOND
        },
        (
            1,
            Some(Primitive::Reach {
                from_hand,
                to_hand,
                ..
            }),
        ) => {
            // Consider Reach Calculation

            // Retrieve data
            let from_hand_info = job.points_of_interest.get(from_hand).unwrap();
            let to_hand_info = job.points_of_interest.get(to_hand).unwrap();

            let motion_vector = to_hand_info.position() - from_hand_info.position();
            let motion_distance = motion_vector.norm();

            return match to_hand_info.variability() {
                Rating::High => {
                    if motion_distance < 0.0254 {
                        2.0 * TMU_PER_SECOND
                    } else if 0.0254 <= motion_distance && motion_distance <= 0.1016 {
                        3.6866*39.37*motion_distance.powf(0.6146) * TMU_PER_SECOND
                    } else if 0.1016 <= motion_distance && motion_distance <= 0.762 {
                        (5.959169 + 0.690797*39.37*motion_distance) * TMU_PER_SECOND
                    } else {
                        (5.959169 + 0.690797*39.37*motion_distance + 0.7*(39.37*motion_distance - 0.762)) * TMU_PER_SECOND
                    }
                },
                Rating::Medium => {
                    if motion_distance < 0.0254 {
                        2.0 * TMU_PER_SECOND
                    } else if 0.0254 <= motion_distance && motion_distance <= 0.1016 {
                        2.5*39.37*motion_distance.powf(0.681) * TMU_PER_SECOND
                    } else if 0.1016 <= motion_distance && motion_distance <= 0.762 {
                        (4.309488 + 0.71666*39.37*motion_distance) * TMU_PER_SECOND
                    } else {
                        (4.309488 + 0.71666*38.37*motion_distance + 0.7*(39.37*motion_distance - 0.762)) * TMU_PER_SECOND
                    }
                },
                Rating::Low => {
                    if motion_distance < 0.0254 {
                        2.0 * TMU_PER_SECOND
                    } else if 0.0254 <= motion_distance && motion_distance <= 0.0762 {
                        2.5*39.37*motion_distance.powf(0.681) * TMU_PER_SECOND
                    } else if 0.0762 <= motion_distance && motion_distance <= 0.762 {
                        (4.333601 + 0.440266*39.37*motion_distance) * TMU_PER_SECOND
                    } else {
                        (4.333601 + 0.440266*39.37*motion_distance + 0.4*(39.37*motion_distance - 0.762)) * TMU_PER_SECOND
                    }
                }
            }
        },
        (
            1,
            Some(Primitive::Force {
                id, 
                target,
                magnitude,
                ..
            }),
        ) => {
            // Consider Force Calculation
            let mut tmu: f64 = 0.0;

            let target_info = job.targets.get(target).unwrap();
            let weight = target_info.weight();

            if *magnitude >= 0.0 {
                // Apply force, dwell minimum, and release
                return 10.6 * TMU_PER_SECOND;
            }

            tmu += 2.0;

            let mut denom = 0.0;
            // assume 2 hands if magnitude is greater than 100 based on the trees
            if *magnitude > 100 {
                // average
                denom += 159;
            } else if *magnitude < -100 {
                // average
                denom += 174;
            } else if *magnitude >= 0 && *magnitude < 100 {
                // average
                denom += 81;
            } else {
                // average
                denom += 92;
            }

            let mut mvc = *magnitude / denom;

            if mvc < 0.2 {
                if weight < 1.13 {
                    tmu += 4.0;
                } else {
                    tmu += 5.7;
                }
            } else if mvc < 0.5 {
                if weight < 1.13 {
                    tmu += 7.5;
                } else {
                    tmu += 11.8;
                }
            } else {
                if weight < 1.13 {
                    tmu += 22.9;
                } else {
                    tmu += 34.7;
                }
            }

            // Convert TMU to seconds
            let time = tmu * TMU_PER_SECOND;

            return time;
        },
        (
            1,
            Some(Primitive::Position {
                id, 
                target,
                ..
            }),
        ) => {
            // Consider Force Calculation
            let mut tmu: f64 = 0.0;

            let target_info = job.targets.get(target).unwrap();
            let weight = target_info.weight();

            // upper bounding by 360 degrees
            if weight < 0.91 {
                // 1.4927 + 0.043878*360
                tmu += 17.28878;
            } else  if weight < 4.54 {
                // 2.3463636 + 0.0689090*360
                tmu += 27.1536036;
            } else {
                // 4.4781818 + 0.131636*360
                tmu += 51.8671418;
            }

            // Convert TMU to seconds
            let time = tmu * TMU_PER_SECOND;

            return time;
        },
        (
            1,
            Some(Primitive::Inspect {
                id, 
                target,
                skill,
                ..
            }),
        ) => {
            // this primitive's time will be based on the time of the primitives coupled with it
            0.0
        },
        (
            1,
            Some(Primitive::Selection {
                id, 
                target,
                skill,
                ..
            }),
        ) => {
            // TODO. look at data to compare hand location to object location
            0.0
        },
        (
            2,
            Some(Primitive::Position {
                id, 
                target,
                ..
            }),
        ) => {
            return match (assigned_primitives.last()) {
                Some(Primitive::Force { id, target, magnitude }) => {
                    let target_info = job.targets.get(target).unwrap();
                    let weight = target_info.weight();
                    let mut tmu = 0.0;

                    // Grasp TMU
                    tmu += 2.0;

                    let mut denom = 0.0;
                    // assume 2 hands if magnitude is greater than 100 based on the trees
                    if *magnitude > 100 {
                        // average
                        denom += 159;
                    } else if *magnitude < -100 {
                        // average
                        denom += 174;
                    } else if *magnitude >= 0 && *magnitude < 100 {
                        // average
                        denom += 81;
                    } else {
                        // average
                        denom += 92;
                    }
        
                    let mut mvc = *magnitude / denom;

                    if mvc < 0.2 {
                        match target_info.symmetry() {
                            Rating::High => {
                                if weight < 1.13 {
                                    tmu += 5.6;
                                } else {
                                    tmu += 11.2;
                                }
                            },
                            Rating::Medium => {
                                if weight < 1.13 {
                                    tmu += 9.1;
                                } else {
                                    tmu += 14.7;
                                }
                            },
                            Rating::Low => {
                                if weight < 1.13 {
                                    tmu += 10.4;
                                } else {
                                    tmu += 16.0;
                                }
                            }
                        }
                    } else if mvc < 0.5 {
                        match target_info.symmetry() {
                            Rating::High => {
                                if weight < 1.13 {
                                    tmu += 16.2;
                                } else {
                                    tmu += 21.8;
                                }
                            },
                            Rating::Medium => {
                                if weight < 1.13 {
                                    tmu += 19.7;
                                } else {
                                    tmu += 25.3;
                                }
                            },
                            Rating::Low => {
                                if weight < 1.13 {
                                    tmu += 21;
                                } else {
                                    tmu += 26.6;
                                }
                            }
                        }
                    } else {
                        match target_info.symmetry() {
                            Rating::High => {
                                if weight < 1.13 {
                                    tmu += 43;
                                } else {
                                    tmu += 48.6;
                                }
                            },
                            Rating::Medium => {
                                if weight < 1.13 {
                                    tmu += 46.5;
                                } else {
                                    tmu += 52.1;
                                }
                            },
                            Rating::Low => {
                                if weight < 1.13 {
                                    tmu += 47.8;
                                } else {
                                    tmu += 53.4;
                                }
                            }
                        }
                    }

                    // release TMU
                    tmu += 2.0;

                    // convert tmu to seconds
                    let time = tmu * TMU_PER_SECOND;

                    return time;
                },
                _ => {
                    0.0
                }
            }
        },
        (
            2,
            Some(Primitive::Force {
                id, 
                target,
                magnitude,
                ..
            }),
        ) => {
            return match (assigned_primitives.last()) {
                Some(Primitive::Position { id, target }) => {
                    let target_info = job.targets.get(target).unwrap();
                    let weight = target_info.weight();
                    let mut tmu = 0.0;

                    // Grasp TMU
                    tmu += 2.0;

                    let mut denom = 0.0;
                    // assume 2 hands if magnitude is greater than 100 based on the trees
                    if *magnitude > 100 {
                        // average
                        denom += 159;
                    } else if *magnitude < -100 {
                        // average
                        denom += 174;
                    } else if *magnitude >= 0 && *magnitude < 100 {
                        // average
                        denom += 81;
                    } else {
                        // average
                        denom += 92;
                    }
        
                    let mut mvc = *magnitude / denom;

                    if mvc < 0.2 {
                        match target_info.symmetry() {
                            Rating::High => {
                                if weight < 1.13 {
                                    tmu += 5.6;
                                } else {
                                    tmu += 11.2;
                                }
                            },
                            Rating::Medium => {
                                if weight < 1.13 {
                                    tmu += 9.1;
                                } else {
                                    tmu += 14.7;
                                }
                            },
                            Rating::Low => {
                                if weight < 1.13 {
                                    tmu += 10.4;
                                } else {
                                    tmu += 16.0;
                                }
                            }
                        }
                    } else if mvc < 0.5 {
                        match target_info.symmetry() {
                            Rating::High => {
                                if weight < 1.13 {
                                    tmu += 16.2;
                                } else {
                                    tmu += 21.8;
                                }
                            },
                            Rating::Medium => {
                                if weight < 1.13 {
                                    tmu += 19.7;
                                } else {
                                    tmu += 25.3;
                                }
                            },
                            Rating::Low => {
                                if weight < 1.13 {
                                    tmu += 21;
                                } else {
                                    tmu += 26.6;
                                }
                            }
                        }
                    } else {
                        match target_info.symmetry() {
                            Rating::High => {
                                if weight < 1.13 {
                                    tmu += 43;
                                } else {
                                    tmu += 48.6;
                                }
                            },
                            Rating::Medium => {
                                if weight < 1.13 {
                                    tmu += 46.5;
                                } else {
                                    tmu += 52.1;
                                }
                            },
                            Rating::Low => {
                                if weight < 1.13 {
                                    tmu += 47.8;
                                } else {
                                    tmu += 53.4;
                                }
                            }
                        }
                    }

                    // release TMU
                    tmu += 2.0;

                    // convert tmu to seconds
                    let time = tmu * TMU_PER_SECOND;

                    return time;
                },
                _ => {
                    0.0
                }
            }
        },
        (_, _) => {
            // There is some non-zero number of assigned primitives. Compute them independently and run the max on them
            0.0
        }
    };
}

#[test]
fn test_grade() {
    let point1 = Vector3::new(0.0, 0.0, 0.0);
    let point2 = Vector3::new(1.0, 1.0, 0.0);
    assert_eq!(get_grade(point1, point2), 0.0);

    let point3 = Vector3::new(0.0, 0.0, 0.0);
    assert_eq!(get_grade(point1, point3), 0.0);

    let point4 = Vector3::new(1.0, 0.0, 1.0);
    assert!((get_grade(point1, point4) - 0.5) < f64::EPSILON);
    assert!((get_grade(point4, point1) - 0.5) < f64::EPSILON);

    let point5 = Vector3::new(1.0, 1.0, 1.0);
    assert_eq!(get_grade(point2, point5), 1.0);

    let point6 = Vector3::new(1.1, 1.1, 0.95);
    assert!(get_grade(point2, point6) > 0.9 && get_grade(point2, point6) < 1.0);

    let point7 = Vector3::new(0.0, 0.0, 1.1);
    assert_eq!(get_grade(point1, point7), 1.0);
}
