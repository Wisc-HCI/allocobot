use crate::constants::{
    DISTANCE_PER_PACE, MAX_ARM_WORK_DISTANCE, MAX_HAND_WORK_DISTANCE, MAX_SHOULDER_WORK_DISTANCE,
    SEC_PER_HOUR, TMU_PER_SECOND,
};
use crate::description::job::Job;
use crate::description::poi::PointOfInterest;
use crate::description::primitive::Primitive;
use crate::description::rating::Rating;
use crate::description::units::Time;
use crate::petri::cost::{Cost, CostCategory, CostFrequency, CostSet};
use crate::petri::data::{Data, DataTag, Query};
use crate::petri::transition::Transition;
use nalgebra::{Vector2, Vector3};
use serde::{Deserialize, Serialize};
use std::{cmp, collections::HashMap, f64::consts::PI};
// use std::collections::HashMap;
use enum_tag::EnumTag;
use uuid::Uuid;
use statrs::distribution::{Normal, ContinuousCDF};
use statrs::statistics::Distribution;

use super::gender::Gender;
use super::target::{self, Target};
use super::units::{TokenCount, Watts, USD};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Agent {
    Robot(RobotInfo),
    Human(HumanInfo),
}

impl Agent {
    pub fn new_robot(
        name: String,
        reach: f64,                   // meters
        vertical_offset: f64,         // meters
        payload: f64,                 // kg
        agility: Rating,              // rating 0-1
        speed: f64,                   // m/s
        precision: f64,               // m (repeatability)
        sensing: Rating,              // rating 0-1
        mobile_speed: f64,            // m/s
        purchase_price: USD,          // dollars
        energy_consumption: Watts,    // watts
        annual_maintenance_cost: USD, //dollars
    ) -> Self {
        return Agent::Robot(RobotInfo {
            id: Uuid::new_v4(),
            name,
            reach,
            vertical_offset,
            payload,
            agility,
            speed,
            precision,
            sensing,
            mobile_speed,
            purchase_price,
            energy_consumption,
            annual_maintenance_cost,
        });
    }

    pub fn new_human(
        name: String,
        age: f64,             // Years
        gender: Gender,
        acromial_height: f64, // meters
        height: f64,          // meters
        reach: f64,           // meters
        weight: f64,          // kg
        skill: Rating,
        hourly_wage: USD,
        labor_cost: USD,
    ) -> Self {
        return Agent::Human(HumanInfo {
            id: Uuid::new_v4(),
            name,
            age,
            gender,
            acromial_height,
            height,
            reach,
            weight,
            skill,
            hourly_wage,
            labor_cost,
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
    pub vertical_offset: f64,
    pub payload: f64,
    pub agility: Rating,
    pub speed: f64,
    pub precision: f64,
    pub sensing: Rating,
    pub mobile_speed: f64,
    pub purchase_price: USD,
    pub energy_consumption: Watts,
    pub annual_maintenance_cost: USD,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HumanInfo {
    pub id: Uuid,
    pub name: String,
    // NOTE: This is not currently used, need to check with Rob
    // pub gender: Gender,
    pub age: f64,
    pub gender: Gender,
    pub acromial_height: f64,
    pub height: f64,
    pub reach: f64,
    pub weight: f64,
    pub skill: Rating,
    pub hourly_wage: USD,
    pub labor_cost: USD,
}

pub trait CostProfiler {
    fn execution_time(&self, transition: &Transition, job: &Job) -> Time;
    fn cost_set(&self, transition: &Transition, job: &Job) -> (CostSet, Vec<Data>);
}

impl CostProfiler for HumanInfo {
    fn execution_time(&self, transition: &Transition, job: &Job) -> Time {
        let assigned_primitives = get_assigned_primitives(transition, job, self.id);

        let mut max_time = 0.0;

        for primitive in assigned_primitives.iter() {
            let temp_vec = vec![*primitive];
            let single_time = get_human_time_for_primitive(temp_vec, transition, job, self);
            if single_time > max_time {
                max_time = single_time;
            }

            for primitive_two in assigned_primitives.iter() {
                let temp_vec = vec![*primitive, *primitive_two];
                let doubles_time = get_human_time_for_primitive(temp_vec, transition, job, self);
                if doubles_time > max_time {
                    max_time = doubles_time;
                }
            }
        }

        return max_time;
    }

    fn cost_set(&self, transition: &Transition, job: &Job) -> (CostSet, Vec<Data>) {
        let assigned_primitives: Vec<&Primitive> = transition
            .meta_data
            .iter()
            .filter(|d| d.tag() == DataTag::PrimitiveAssignment && d.id() == Some(self.id))
            .map(|d| job.primitives.get(&d.secondary().unwrap()).unwrap())
            .collect();

        let mut ergo_cost_set = CostSet::new();

        let execution_time = self.execution_time(transition, job);
        if execution_time > 0.0 {
            ergo_cost_set.push(Cost {
                frequency: CostFrequency::Extrapolated,
                value: self.hourly_wage * execution_time / SEC_PER_HOUR,
                category: CostCategory::Monetary,
            });
        }

        let force_magnitude_on_target: HashMap<Uuid, f64> = assigned_primitives
            .iter()
            .filter_map(|p| match p {
                Primitive::Force {
                    magnitude, target, ..
                } => Some((*target, *magnitude)),
                _ => None,
            })
            .collect();

        if transition.has_data(&vec![Query::Data(Data::AgentAdd(self.id))]) {
            ergo_cost_set.push(Cost {
                frequency: CostFrequency::Once,
                value: self.labor_cost,
                category: CostCategory::Monetary,
            });
        }

        for data in transition.meta_data.iter() {
            match *data {
                Data::Spawn(_target_id, cost) => {
                    ergo_cost_set.push(Cost {
                        frequency: CostFrequency::Extrapolated,
                        value: cost,
                        category: CostCategory::Monetary,
                    });
                }
                Data::Produce(_target_id, cost) => {
                    ergo_cost_set.push(Cost {
                        frequency: CostFrequency::Extrapolated,
                        value: -cost,
                        category: CostCategory::Monetary,
                    });
                }
                _ => {}
            }
        }

        let mut new_ergo_meta_data: Vec<Data> = Vec::new();

        for primitive in assigned_primitives.iter() {
            // for primitive in &assigned_primitives {
            match *primitive {
                Primitive::Force {
                    target,
                    magnitude,
                    id,
                    ..
                } => {
                    let target_info = job.targets.get(target).unwrap();
                    let weight = target_info.weight();

                    let (mvc, hand_to_floor_dist, dist, is_one_hand) =
                        get_force_mvc(transition, magnitude, self, job, weight);
                    let cost = mvc * execution_time;

                    ergo_cost_set.push(Cost {
                        frequency: CostFrequency::Extrapolated,
                        value: cost,
                        category: CostCategory::Ergonomic,
                    });

                    let mut new_data = vec_ergo_meta_data(self, dist, cost);
                    new_ergo_meta_data.append(&mut new_data);
                    new_ergo_meta_data.push(Data::HandDistanceToFloor(*id, hand_to_floor_dist));
                    new_ergo_meta_data.push(Data::MVC(*id, mvc));
                    new_ergo_meta_data
                        .push(Data::IsOneHanded(*id, if is_one_hand { 1.0 } else { 0.0 }));
                }
                Primitive::Carry {
                    target,
                    to_standing,
                    from_standing,
                    to_hand,
                    from_hand,
                    id,
                    ..
                } => {
                    let target_info = job.targets.get(target).unwrap();
                    let weight = target_info.weight();
                    let (hand_location, stand_location) =
                        get_hand_stand_locations(transition, self, job);
                    let is_one_hand = is_one_hand_task(hand_location, stand_location, weight);

                    let from_standing_info = job.points_of_interest.get(from_standing).unwrap();
                    let to_standing_info = job.points_of_interest.get(to_standing).unwrap();
                    let to_hand_info = job.points_of_interest.get(to_hand).unwrap();
                    let from_hand_info = job.points_of_interest.get(from_hand).unwrap();

                    let mut to_shoulder_pos = to_standing_info.position().clone();
                    to_shoulder_pos.z = to_shoulder_pos.z + self.acromial_height;
                    let mut from_shoulder_pos = from_standing_info.position().clone();
                    from_shoulder_pos.z = from_shoulder_pos.z + self.acromial_height;
                    let hand_travel_vector = to_hand_info.position() - to_shoulder_pos;
                    let hand_travel_distance = hand_travel_vector.norm();
                    let total_distance_traveled = (to_standing_info.position().clone() - from_standing_info.position().clone())
                    .norm();

                    let mut denom = 0.0;
                    if is_one_hand {
                        if hand_travel_distance < 0.5*self.reach {
                            if self.gender == Gender::Female {
                                let n = Normal::new(105.0, 32.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            } else {
                                let n = Normal::new(232.5, 54.5).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            }
                        } else if hand_travel_distance < 0.75*self.reach {
                            if self.gender == Gender::Female {
                                let n = Normal::new(73.5, 25.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            } else {
                                let n = Normal::new(118.5, 25.5).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            }
                        } else if hand_travel_distance <= self.reach {
                            
                            if self.gender == Gender::Female {
                                let n = Normal::new(46.0, 13.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            } else {
                                let n = Normal::new(75.5, 13.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            }
                        } else {
                            denom += 0.001;
                        }
                    } else {
                        if hand_travel_distance < 0.5*self.reach {
                            if self.gender == Gender::Female {
                                let n = Normal::new(210.0, 64.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            } else {
                                let n = Normal::new(465.0, 109.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            }
                        } else if hand_travel_distance < 0.75*self.reach {
                            if self.gender == Gender::Female {
                                let n = Normal::new(147.0, 50.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            } else {
                                let n = Normal::new(237.0, 51.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            }
                        } else if hand_travel_distance <= self.reach {
                            if self.gender == Gender::Female {
                                let n = Normal::new(92.0, 26.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            } else {
                                let n = Normal::new(151.0, 26.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            }
                        } else {
                            denom += 0.001;
                        }
                    }

                    let mvc = weight / denom;
                    let cost = mvc * execution_time;
                    ergo_cost_set.push(Cost {
                        frequency: CostFrequency::Extrapolated,
                        value: cost,
                        category: CostCategory::Ergonomic,
                    });

                    new_ergo_meta_data.push(Data::ErgoArm(self.id, cost));
                    new_ergo_meta_data.push(Data::ReachDistance(*id, hand_travel_distance));
                    new_ergo_meta_data
                        .push(Data::StandTravelDistance(*id, total_distance_traveled));
                    new_ergo_meta_data.push(Data::MVC(*id, mvc));
                    new_ergo_meta_data
                        .push(Data::IsOneHanded(*id, if is_one_hand { 1.0 } else { 0.0 }));
                }
                Primitive::Move {
                    target,
                    standing,
                    from_hand,
                    to_hand,
                    id,
                    ..
                } => {
                    let to_hand_info = job.points_of_interest.get(to_hand).unwrap();
                    let from_hand_info = job.points_of_interest.get(from_hand).unwrap();
                    let standing_info = job.points_of_interest.get(standing).unwrap();

                    let target_info = job.targets.get(target).unwrap();
                    let weight = target_info.weight();

                    let to_hand_pos = to_hand_info.position();
                    let from_hand_pos = from_hand_info.position();
                    let standing_pos = standing_info.position();

                    let is_one_hand = is_one_hand_task(to_hand_pos, standing_pos, weight);

                    let horizontal_distance = (Vector2::new(to_hand_pos.x, to_hand_pos.y)
                        - Vector2::new(from_hand_pos.x, from_hand_pos.y))
                    .norm();
                    let horizontal_hand_shoulder_distance =
                        (Vector2::new(to_hand_pos.x, to_hand_pos.y)
                            - Vector2::new(standing_pos.x, standing_pos.y))
                        .norm();
                    let vertical_distance = to_hand_pos.z - from_hand_pos.z;

                    let is_arm_work_bool = is_arm_work(horizontal_hand_shoulder_distance);

                    // todo: might need to offset z by the acromial height
                    let mut shoulder_pos = standing_pos.clone();
                    shoulder_pos.z = shoulder_pos.z + self.acromial_height;
                    let reach_distance = (shoulder_pos - to_hand_pos).norm();

                    let hand_distance_to_floor = to_hand_pos.z - standing_pos.z;

                    let mut denom = 0.0;
                    let mut vertical_distance_is_zero = false;

                    if vertical_distance == 0.0 {
                        vertical_distance_is_zero = true;
                    } else {
                        if is_one_hand {
                            if horizontal_hand_shoulder_distance < 0.45 {
                                if reach_distance < 0.5*self.reach {
                                    if self.gender == Gender::Female {
                                        let n = Normal::new(130.5, 43.5).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    } else {
                                        let n = Normal::new(295.0, 70.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    }
                                } else if reach_distance < 0.75*self.reach {
                                    if self.gender == Gender::Female {
                                        let n = Normal::new(91.0, 31.5).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    } else {
                                        let n = Normal::new(147.5, 33.5).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    }
                                } else {
                                    if self.gender == Gender::Female {
                                        let n = Normal::new(59.0, 16.5).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    } else {
                                        let n = Normal::new(92.0, 17.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    }
                                }
                            } else if horizontal_hand_shoulder_distance < 2.0 {
                                if reach_distance < 0.5*self.reach {
                                    if self.gender == Gender::Female {
                                        let n = Normal::new(87.5, 29.5).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    } else {
                                        let n = Normal::new(192.0, 48.5).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    }
                                } else if reach_distance < 0.75*self.reach {
                                    if self.gender == Gender::Female {
                                        let n = Normal::new(68.5, 17.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    } else {
                                        let n = Normal::new(145.0, 45.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    }
                                } else {
                                    if self.gender == Gender::Female {
                                        let n = Normal::new(46.5, 9.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    } else {
                                        let n = Normal::new(99.5, 39.5).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    }
                                }
                            } else {
                                denom += 0.001;
                            }
                        } else {
                            if horizontal_hand_shoulder_distance < 0.45 {
                                if reach_distance < 0.5*self.reach {
                                    if self.gender == Gender::Female {
                                        let n = Normal::new(261.0, 87.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    } else {
                                        let n = Normal::new(590.0, 140.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    }
                                } else if reach_distance < 0.75*self.reach {
                                    if self.gender == Gender::Female {
                                        let n = Normal::new(182.0, 63.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    } else {
                                        let n = Normal::new(295.0, 67.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    }
                                } else {
                                    if self.gender == Gender::Female {
                                        let n = Normal::new(118.0, 33.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    } else {
                                        let n = Normal::new(184.0, 34.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    }
                                }
                            } else if horizontal_hand_shoulder_distance < 2.0 {
                                if reach_distance < 0.5*self.reach {
                                    if self.gender == Gender::Female {
                                        let n = Normal::new(175.0, 59.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    } else {
                                        let n = Normal::new(384.0, 97.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    }
                                } else if reach_distance < 0.75*self.reach {
                                    if self.gender == Gender::Female {
                                        let n = Normal::new(137.0, 59.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    } else {
                                        let n = Normal::new(290.0, 90.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    }
                                } else {
                                    if self.gender == Gender::Female {
                                        let n = Normal::new(93.0, 18.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    } else {
                                        let n = Normal::new(199.0, 79.0).unwrap();
                                        denom += n.inverse_cdf(job.target_pop);
                                    }
                                }
                            } else {
                                denom += 0.001;
                            }
                        }
                    
                    }

                    let mut mvc = 0.0;
                    if vertical_distance_is_zero {
                        // TODO: no magnitude here....... what do??????
                        // Todo: 
                        let (force_mvc, _hand_to_floor_dist, _dist, _is_one_hand) = get_force_mvc(transition, &horizontal_distance, self, job, weight);
                        mvc += force_mvc;
                    } else {
                        mvc += weight / denom;
                    }

                    let cost = mvc * execution_time;

                    ergo_cost_set.push(Cost {
                        frequency: CostFrequency::Extrapolated,
                        value: cost,
                        category: CostCategory::Ergonomic,
                    });

                    let mut new_data = vec_ergo_meta_data(self, horizontal_distance, cost);
                    new_ergo_meta_data.append(&mut new_data);
                    new_ergo_meta_data.push(Data::HorizontalHandTravelDistance(*id, horizontal_distance));
                    new_ergo_meta_data.push(Data::VerticalHandTravelDistance(*id, vertical_distance));
                    new_ergo_meta_data.push(Data::ReachDistance(*id, reach_distance));
                    new_ergo_meta_data.push(Data::HandDistanceToFloor(*id, hand_distance_to_floor));
                    new_ergo_meta_data.push(Data::MVC(*id, mvc));
                    new_ergo_meta_data
                        .push(Data::IsOneHanded(*id, if is_one_hand { 1.0 } else { 0.0 }));
                }
                Primitive::Use { target, id, .. } => {
                    let target_info = job.targets.get(target).unwrap();
                    let size = target_info.size();
                    let weight = target_info.weight();
                    // calculate volume of sphere based on the size
                    let volume = 4.0 / 3.0 * PI * f64::powf(size, 3.0);

                    let (hand_location, stand_location) =
                        get_hand_stand_locations(transition, self, job);
                    let horizontal_hand_shoulder_distance =
                        (Vector2::new(hand_location.x, hand_location.y)
                            - Vector2::new(stand_location.x, stand_location.y))
                        .norm();

                    let mut denom = 0.0;
                    if volume > 0.406 {
                        if self.gender == Gender::Female {
                            let n = Normal::new(308.0, 61.2).unwrap();
                            denom += n.inverse_cdf(job.target_pop);
                        } else {
                            let n = Normal::new(487.5, 109.1).unwrap();
                            denom += n.inverse_cdf(job.target_pop);
                        }
                    } else {
                        if self.gender == Gender::Female {
                            let n = Normal::new(68.6, 12.2).unwrap();
                            denom += n.inverse_cdf(job.target_pop);
                        } else {
                            let n = Normal::new(92.2, 13.6).unwrap();
                            denom += n.inverse_cdf(job.target_pop);
                        }
                    }

                    let mvc = weight / denom;
                    let cost = mvc * execution_time;

                    ergo_cost_set.push(Cost {
                        frequency: CostFrequency::Extrapolated,
                        value: cost,
                        category: CostCategory::Ergonomic,
                    });

                    let mut new_data =
                        vec_ergo_meta_data(self, horizontal_hand_shoulder_distance, cost);
                    new_ergo_meta_data.append(&mut new_data);
                    new_ergo_meta_data.push(Data::HorizontalHandTravelDistance(*id, horizontal_hand_shoulder_distance));
                    new_ergo_meta_data.push(Data::MVC(*id, mvc));
                }
                Primitive::Travel {
                    id,
                    from_standing,
                    to_standing,
                    from_hand,
                    to_hand,
                    ..
                } => {
                    // Consider Travel Calculation

                    // Retrieve data
                    let from_standing_info = job.points_of_interest.get(from_standing).unwrap();
                    let to_standing_info = job.points_of_interest.get(to_standing).unwrap();

                    // Compute travel vector/distance
                    let travel_vector = to_standing_info.position() - from_standing_info.position();
                    let travel_distance = travel_vector.norm();
                    new_ergo_meta_data.push(Data::StandTravelDistance(*id, travel_distance));
                }
                Primitive::Hold { target, id, .. } => {
                    let target_info = job.targets.get(target).unwrap();
                    let weight = target_info.weight();
                    let (hand_location, stand_location) =
                        get_hand_stand_locations(transition, self, job);
                    let is_one_hand = is_one_hand_task(hand_location, stand_location, weight);

                    let horizontal_hand_shoulder_distance =
                        (Vector2::new(hand_location.x, hand_location.y)
                            - Vector2::new(stand_location.x, stand_location.y))
                        .norm();
                    let is_hand_work = horizontal_hand_shoulder_distance < MAX_HAND_WORK_DISTANCE;

                    let mut denom = 0.0;

                    // todo: shoulder and elbow angles????
                    if is_one_hand {
                        if is_hand_work {
                             // elbow at 90 degrees

                            if self.gender == Gender::Female {
                                let n = Normal::new(105.0, 32.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            } else {
                                let n = Normal::new(232.5, 54.5).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            }
                        } else {
                            // shoulder at 90 degrees

                            if self.gender == Gender::Female {
                                let n = Normal::new(46.0, 13.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            } else {
                                let n = Normal::new(75.5, 13.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            }
                        }
                    } else {
                        if is_hand_work {
                            // elbow at 90  degrees

                            if self.gender == Gender::Female {
                                let n = Normal::new(210.0, 64.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            } else {
                                let n = Normal::new(465.0, 109.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            }
                        } else {
                            // shoulder at 90 degrees

                            if self.gender == Gender::Female {
                                let n = Normal::new(92.0, 26.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            } else {
                                let n = Normal::new(151.0, 26.0).unwrap();
                                denom += n.inverse_cdf(job.target_pop);
                            }
                        }
                    }
                    let mvc = weight / denom;
                    let cost = mvc * execution_time;

                    ergo_cost_set.push(Cost {
                        frequency: CostFrequency::Extrapolated,
                        value: cost,
                        category: CostCategory::Ergonomic,
                    });

                    let mut new_data =
                        vec_ergo_meta_data(self, horizontal_hand_shoulder_distance, cost);
                    new_ergo_meta_data.append(&mut new_data);
                    new_ergo_meta_data.push(Data::ReachDistance(*id, horizontal_hand_shoulder_distance));
                    new_ergo_meta_data.push(Data::MVC(*id, mvc));
                    new_ergo_meta_data
                        .push(Data::IsOneHanded(*id, if is_one_hand { 1.0 } else { 0.0 }));
                    new_ergo_meta_data.push(Data::IsHandWork(*id, if is_hand_work { 1.0 } else { 0.0 }));
                }
                _ => {}
            }
        }

        (ergo_cost_set, new_ergo_meta_data)
    }
}

impl CostProfiler for RobotInfo {
    fn execution_time(&self, transition: &Transition, job: &Job) -> Time {
        let assigned_primitives = get_assigned_primitives(transition, job, self.id);

        let mut max_time = 0.0;

        for primitive in assigned_primitives.iter() {
            let temp_vec = vec![*primitive];
            let single_time = get_robot_time_for_primitive(temp_vec, transition, job, self);
            if single_time > max_time {
                max_time = single_time;
            }

            for primitive_two in assigned_primitives.iter() {
                let temp_vec = vec![*primitive, *primitive_two];
                let doubles_time = get_robot_time_for_primitive(temp_vec, transition, job, self);
                if doubles_time > max_time {
                    max_time = doubles_time;
                }
            }
        }

        return max_time;
    }

    fn cost_set(&self, transition: &Transition, job: &Job) -> (CostSet, Vec<Data>) {
        let assigned_primitives: Vec<&Primitive> = transition
            .meta_data
            .iter()
            .filter(|d| d.tag() == DataTag::PrimitiveAssignment && d.id() == Some(self.id))
            .map(|d| job.primitives.get(&d.secondary().unwrap()).unwrap())
            .collect();

        let mut robot_cost_set = CostSet::new();
        
        // Add one-time purchasing cost (if the transition adds the agent)
        if transition.has_data(&vec![Query::Data(Data::AgentAdd(self.id))]) {
            robot_cost_set.push(Cost {
                frequency: CostFrequency::Once,
                value: self.purchase_price,
                category: CostCategory::Monetary,
            });
        }

        // Add electricity cost
        let execution_time = self.execution_time(transition, job);
        if execution_time > 0.0 {
            robot_cost_set.push(Cost {
                frequency: CostFrequency::Extrapolated,
                value: (self.energy_consumption * execution_time / SEC_PER_HOUR) * job.kwh_cost, // cost is $/kWh
                category: CostCategory::Monetary,
            });
        }

        let max_error_cost = get_produced_value(job);

        // Cost for integration
        // Cost for error
        for primitive in assigned_primitives.iter() {
            match *primitive {
                Primitive::Carry {
                    id,
                    target,
                    from_standing,
                    to_standing,
                    from_hand,
                    to_hand,
                } => {
                    let to_hand_info = job.points_of_interest.get(to_hand).unwrap();
                    let cost = get_robot_error_rate_independent(to_hand_info.structure(), to_hand_info.variability(), self.sensing.clone()) * max_error_cost;
                    // let cost = get_robot_error_rate_exponential(to_hand_info.structure(), to_hand_info.variability(), self.sensing);
                    // let cost = get_robot_error_rate_multiplicative(to_hand_info.structure(), to_hand_info.variability(), self.sensing);
                    
                    // TODO: integration cost

                    // error cost
                    robot_cost_set.push(Cost {
                        frequency: CostFrequency::Extrapolated,
                        value: cost,
                        category: CostCategory::Monetary,
                    });
                }
                Primitive::Move {
                    id,
                    target,
                    standing,
                    from_hand,
                    to_hand,
                } => {
                    let to_hand_info = job.points_of_interest.get(to_hand).unwrap();
                    let cost = get_robot_error_rate_independent(to_hand_info.structure(), to_hand_info.variability(), self.sensing.clone()) * max_error_cost;
                    // let cost = get_robot_error_rate_exponential(to_hand_info.structure(), to_hand_info.variability(), self.sensing);
                    // let cost = get_robot_error_rate_multiplicative(to_hand_info.structure(), to_hand_info.variability(), self.sensing);
                    
                    // TODO: integration cost

                    // error cost
                    robot_cost_set.push(Cost {
                        frequency: CostFrequency::Extrapolated,
                        value: cost,
                        category: CostCategory::Monetary,
                    });
                }
                Primitive::Travel {
                    id,
                    from_standing,
                    to_standing,
                    from_hand,
                    to_hand,
                } => {
                    let to_hand_info = job.points_of_interest.get(to_hand).unwrap();
                    let cost = get_robot_error_rate_independent(to_hand_info.structure(), to_hand_info.variability(), self.sensing.clone()) * max_error_cost;
                    // let cost = get_robot_error_rate_exponential(to_hand_info.structure(), to_hand_info.variability(), self.sensing);
                    // let cost = get_robot_error_rate_multiplicative(to_hand_info.structure(), to_hand_info.variability(), self.sensing);
                    
                    // TODO: integration cost

                    // error cost
                    robot_cost_set.push(Cost {
                        frequency: CostFrequency::Extrapolated,
                        value: cost,
                        category: CostCategory::Monetary,
                    });
                }
                Primitive::Inspect { skill, ..} => {
                    let mut cost = 0.0;
                    if *skill == Rating::High && self.sensing == Rating::High {
                        cost += 0.01;
                    } else if *skill == Rating::High && self.sensing == Rating::Medium {
                        cost += 0.125;
                    } else if *skill == Rating::High && self.sensing == Rating::Low {
                        cost += 0.25;
                    } else if *skill == Rating::Medium && self.sensing == Rating::High {
                        cost += 0.0075;
                    } else if *skill == Rating::Medium && self.sensing == Rating::Medium {
                        cost += 0.06;
                    } else if *skill == Rating::Medium && self.sensing == Rating::Low {
                        cost += 0.1;
                    } else if *skill == Rating::Low && self.sensing == Rating::High {
                        cost += 0.005;
                    } else if *skill == Rating::Low && self.sensing == Rating::Medium {
                        cost += 0.015;
                    } else if *skill == Rating::Low && self.sensing == Rating::Low {
                        cost += 0.05;
                    }
                    cost *= max_error_cost;

                    // TODO: integration cost

                    // error cost
                    robot_cost_set.push(Cost {
                        frequency: CostFrequency::Extrapolated,
                        value: cost,
                        category: CostCategory::Monetary,
                    });
                }
                Primitive::Selection { skill, ..} => {
                    
                    let mut cost = 0.0;
                    if *skill == Rating::High && self.sensing == Rating::High {
                        cost += 0.01;
                    } else if *skill == Rating::High && self.sensing == Rating::Medium {
                        cost += 0.125;
                    } else if *skill == Rating::High && self.sensing == Rating::Low {
                        cost += 0.25;
                    } else if *skill == Rating::Medium && self.sensing == Rating::High {
                        cost += 0.0075;
                    } else if *skill == Rating::Medium && self.sensing == Rating::Medium {
                        cost += 0.06;
                    } else if *skill == Rating::Medium && self.sensing == Rating::Low {
                        cost += 0.1;
                    } else if *skill == Rating::Low && self.sensing == Rating::High {
                        cost += 0.005;
                    } else if *skill == Rating::Low && self.sensing == Rating::Medium {
                        cost += 0.015;
                    } else if *skill == Rating::Low && self.sensing == Rating::Low {
                        cost += 0.05;
                    }
                    cost *= max_error_cost;

                    // TODO: integration cost

                    // error cost
                    robot_cost_set.push(Cost {
                        frequency: CostFrequency::Extrapolated,
                        value: cost,
                        category: CostCategory::Monetary,
                    });
                }
                _ => {

                }
            }
        }

        (robot_cost_set, vec![])
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

fn get_produced_value(job: &Job) -> f64 {
    let mut total_value = 0.0;
    for (id, target) in job.targets.iter() {
        if let Target::Product { id, name, size, weight, symmetry, pois, value } = target {
            total_value += target.value();
        }
    }

    return total_value;
}

fn robot_error_rate_base(structure: Rating, variability: Rating) -> f64 {
    if structure == Rating::High && variability == Rating::High {
        return 0.6;
    } else if structure == Rating::High && variability == Rating::Medium {
        return 0.3;
    } else if structure == Rating::High && variability == Rating::Low {
        return 0.1;
    } else if structure == Rating::Medium && variability == Rating::High {
        return 0.75;
    } else if structure == Rating::Medium && variability == Rating::Medium {
        return 0.5;
    } else if structure == Rating::Medium && variability == Rating::Low {
        return 0.25;
    } else if structure == Rating::Low && variability == Rating::High {
        return 1.0;
    } else if structure == Rating::Low && variability == Rating::Medium {
        return 0.75;
    } else {
        return 0.4;
    }
}

fn get_robot_error_rate_multiplicative(structure: Rating, variability: Rating, sensing: Rating) -> f64 {
    let base_value = robot_error_rate_base(structure, variability);

    if sensing == Rating::Low {
        return base_value;
    } else if sensing == Rating::Medium {
        return 0.1 * base_value;
    }

    return 0.01 * base_value;
}

fn rating_to_f64(rating: Rating) -> f64 {
    if rating == Rating::Low {
        return 3.0;
    } else if rating == Rating::Medium {
        return 2.0;
    }

    return 1.0;
}

fn get_robot_error_rate_exponential(structure: Rating, variability: Rating, sensing: Rating) -> f64 {
    return ((-4.0+rating_to_f64(variability)).exp()) + (-rating_to_f64(structure)).exp() + (-rating_to_f64(sensing)).exp();
}

fn get_robot_error_rate_independent(structure: Rating, variability: Rating, sensing: Rating) -> f64 {
    if sensing == Rating::Low {
        if structure == Rating::High && variability == Rating::High {
            return 0.6;
        } else if structure == Rating::High && variability == Rating::Medium {
            return 0.3;
        } else if structure == Rating::High && variability == Rating::Low {
            return 0.1;
        } else if structure == Rating::Medium && variability == Rating::High {
            return 0.75;
        } else if structure == Rating::Medium && variability == Rating::Medium {
            return 0.5;
        } else if structure == Rating::Medium && variability == Rating::Low {
            return 0.25;
        } else if structure == Rating::Low && variability == Rating::High {
            return 1.0;
        } else if structure == Rating::Low && variability == Rating::Medium {
            return 0.75;
        } else {
            return 0.4;
        }
    } else if sensing == Rating::Medium {
        if structure == Rating::High && variability == Rating::High {
            return 0.6;
        } else if structure == Rating::High && variability == Rating::Medium {
            return 0.3;
        } else if structure == Rating::High && variability == Rating::Low {
            return 0.1;
        } else if structure == Rating::Medium && variability == Rating::High {
            return 0.75;
        } else if structure == Rating::Medium && variability == Rating::Medium {
            return 0.5;
        } else if structure == Rating::Medium && variability == Rating::Low {
            return 0.25;
        } else if structure == Rating::Low && variability == Rating::High {
            return 1.0;
        } else if structure == Rating::Low && variability == Rating::Medium {
            return 0.75;
        } else {
            return 0.4;
        }
    } else {
        
        if structure == Rating::High && variability == Rating::High {
            return 0.006;
        } else if structure == Rating::High && variability == Rating::Medium {
            return 0.003;
        } else if structure == Rating::High && variability == Rating::Low {
            return 0.001;
        } else if structure == Rating::Medium && variability == Rating::High {
            return 0.75;
        } else if structure == Rating::Medium && variability == Rating::Medium {
            return 0.5;
        } else if structure == Rating::Medium && variability == Rating::Low {
            return 0.25;
        } else if structure == Rating::Low && variability == Rating::High {
            return 1.0;
        } else if structure == Rating::Low && variability == Rating::Medium {
            return 0.75;
        } else {
            return 0.4;
        }
    }
}

const HUMAN_WEIGHT_FACTORS_MAPPING: [(f64, f64, f64, f64); 9] = [
    // (LOWER BOUND, UPPER BOUND, FACTOR, CONSTANT)
    (11.0853, 33.354, 1.06, 2.2),
    (33.354, 55.6227, 1.11, 3.9),
    (55.6227, 77.8914, 1.17, 5.6),
    (77.8914, 100.1601, 1.22, 7.4),
    (100.1601, 122.625, 1.28, 9.1),
    (122.625, 144.5994, 1.33, 10.8),
    (144.5994, 166.77, 1.39, 12.5),
    (166.77, 188.9406, 1.44, 14.3),
    (188.9406, f64::INFINITY, 1.5, 16.0),
];

fn get_assigned_primitives<'t>(
    transition: &'t Transition,
    job: &'t Job,
    agent_id: Uuid,
) -> Vec<&'t Primitive> {
    transition
        .meta_data
        .iter()
        .filter(|d| d.tag() == DataTag::PrimitiveAssignment && d.id() == Some(agent_id))
        .map(|d| job.primitives.get(&d.secondary().unwrap()).unwrap())
        .collect()
}

fn get_is_within_neutral_reach(
    standing_poi: &PointOfInterest,
    hand_poi: &PointOfInterest,
    acromial_height: f64,
    reach: f64,
) -> bool {
    let standing_vector = standing_poi.position();
    let hand_vector = hand_poi.position();
    let acromial_vector = Vector3::new(0.0, 0.0, acromial_height);
    let acromial_standing_vector = standing_vector + acromial_vector;
    let distance = (hand_vector - acromial_standing_vector).norm();
    return distance <= reach;
}

fn is_arm_work(horizontal_distance: f64) -> bool {
    return horizontal_distance < 0.45;
}

fn is_one_hand_task(
    hand_location: Vector3<f64>,
    stand_location: Vector3<f64>,
    weight: f64,
) -> bool {
    // Check whether this is a 1 or 2 handed activity
    let mut is_one_hand = true;
    let horizontal_distance = (Vector2::new(hand_location.x, hand_location.y)
        - Vector2::new(stand_location.x, stand_location.y))
    .norm();
    if horizontal_distance < 0.05 && weight > 9.81 {
        is_one_hand = false;
    } else if horizontal_distance < 0.15 && weight > 39.24 {
        is_one_hand = false;
    } else if horizontal_distance < 0.45 && weight > 78.48 {
        is_one_hand = false;
    } else if horizontal_distance < 2.0 {
        is_one_hand = false;
    }
    return is_one_hand;
}

fn get_hand_stand_locations(
    transition: &Transition,
    agent: &HumanInfo,
    job: &Job,
) -> (Vector3<f64>, Vector3<f64>) {
    let mut hand_location = Vector3::new(0.0, 0.0, 0.0);
    let mut stand_location = Vector3::new(0.0, 0.0, 0.0);

    for data in transition.meta_data.iter() {
        match data {
            Data::Hand(poi_id, agent_id) => {
                if *agent_id == agent.id {
                    let hand_poi = job.points_of_interest.get(poi_id).unwrap();
                    hand_location = hand_poi.position().clone();
                }
            }
            Data::Standing(poi_id, agent_id) => {
                if *agent_id == agent.id {
                    let stand_poi = job.points_of_interest.get(poi_id).unwrap();
                    stand_location = stand_poi.position().clone();
                }
            }
            _ => {}
        }
    }
    return (hand_location, stand_location);
}

fn get_force_mvc(
    transition: &Transition,
    magnitude: &f64,
    agent: &HumanInfo,
    job: &Job,
    weight: f64,
) -> (f64, f64, f64, bool) {
    let (hand_location, stand_location) = get_hand_stand_locations(transition, agent, job);

    let is_one_hand = is_one_hand_task(hand_location, stand_location, weight);

    // TODO: use vertical offset
    let hand_distance_to_floor = hand_location.z - stand_location.z;
    let horizontal_hand_shoulder_distance = (Vector2::new(hand_location.x, hand_location.y)
        - Vector2::new(stand_location.x, stand_location.y))
    .norm();

    let mut denom = 0.0;
    if !is_one_hand && *magnitude >= 0.0 {
        if hand_distance_to_floor < 0.5 {
            if agent.gender == Gender::Female {
                let n = Normal::new(246.0, 51.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            } else {
                let n = Normal::new(362.0, 112.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            }
        } else if hand_distance_to_floor < 1.0 {
            if agent.gender == Gender::Female {
                let n = Normal::new(339.0, 76.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            } else {
                let n = Normal::new(520.0, 174.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            }
        } else {
            if agent.gender == Gender::Female {
                let n = Normal::new(272.0, 68.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            } else {
                let n = Normal::new(482.0, 165.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            }
        }
    } else if !is_one_hand && *magnitude < 0.0 {
        if hand_distance_to_floor < 0.5 {
            if agent.gender == Gender::Female {
                let n = Normal::new(209.0, 82.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            } else {
                let n = Normal::new(356.0, 61.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            }
        } else if hand_distance_to_floor < 1.0 {
            if agent.gender == Gender::Female {
                let n = Normal::new(521.0, 95.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            } else {
                let n = Normal::new(763.0, 202.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            }
        } else {
            if agent.gender == Gender::Female {
                let n = Normal::new(427.0, 76.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            } else {
                let n = Normal::new(744.0, 243.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            }
        }
    } else if *magnitude >= 0.0 {
        if hand_distance_to_floor < ((1.0 - 0.336)*agent.height) {
            if agent.gender == Gender::Female {
                let n = Normal::new(108.0, 18.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            } else {
                let n = Normal::new(147.0, 25.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            }
        } else if hand_distance_to_floor < ((1.0 - 0.182)*agent.height) {
            if agent.gender == Gender::Female {
                let n = Normal::new(107.0, 16.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            } else {
                let n = Normal::new(136.0, 26.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            }
        } else {
            if agent.gender == Gender::Female {
                let n = Normal::new(144.0, 25.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            } else {
                let n = Normal::new(201.0, 53.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            }
        }
    } else {
        if hand_distance_to_floor < ((1.0 - 0.336)*agent.height) {
            if agent.gender == Gender::Female {
                let n = Normal::new(128.0, 27.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            } else {
                let n = Normal::new(168.0, 33.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            }
        } else if hand_distance_to_floor < ((1.0 - 0.182)*agent.height) {
            if agent.gender == Gender::Female {
                let n = Normal::new(122.0, 29.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            } else {
                let n = Normal::new(145.0, 25.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            }
        } else {
            if agent.gender == Gender::Female {
                let n = Normal::new(178.0, 40.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            } else {
                let n = Normal::new(241.0, 61.0).unwrap();
                denom += n.inverse_cdf(job.target_pop);
            }
        }
    }

    let mvc = (*magnitude).abs() / denom;
    return (mvc, hand_distance_to_floor, horizontal_hand_shoulder_distance, is_one_hand);
}

fn vec_ergo_meta_data(agent: &HumanInfo, dist: f64, cost: f64) -> Vec<Data> {
    let mut result: Vec<Data> = Vec::new();
    if dist < MAX_HAND_WORK_DISTANCE {
        result.push(Data::ErgoHand(agent.id, cost));
    } else if dist < MAX_ARM_WORK_DISTANCE {
        result.push(Data::ErgoArm(agent.id, cost));
    } else if dist < MAX_SHOULDER_WORK_DISTANCE {
        result.push(Data::ErgoShoulder(agent.id, cost));
    } else {
        result.push(Data::ErgoWholeBody(agent.id, cost));
    }
    return result;
}

fn get_human_time_for_primitive(
    assigned_primitives: Vec<&Primitive>,
    transition: &Transition,
    job: &Job,
    agent: &HumanInfo,
) -> Time {
    return match (assigned_primitives.len(), assigned_primitives.first()) {
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
            if get_is_within_neutral_reach(
                from_standing_info,
                from_hand_info,
                agent.acromial_height,
                agent.reach,
            ) {
                tmu += 30.5;
            }

            // Compute travel vector/distance
            let travel_vector = to_standing_info.position() - from_standing_info.position();
            let travel_distance = travel_vector.norm();
            tmu += 17.0 * (travel_distance / 1.19);

            // If the target hand is below the reachable area, based on standing location, add a tmu penalty
            if get_is_within_neutral_reach(
                to_standing_info,
                to_hand_info,
                agent.acromial_height,
                agent.reach,
            ) {
                tmu += 30.5;
            }

            // Release time
            tmu += 2.0;

            // Convert from TMU to seconds
            let time = tmu * TMU_PER_SECOND;

            return time;
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
            tmu += 2.0;

            // If the source hand is below the reachable area, based on standing location, add a tmu penalty
            if get_is_within_neutral_reach(
                standing_info,
                from_hand_info,
                agent.acromial_height,
                agent.reach,
            ) {
                tmu += 30.5;
            }

            // Movement tmu
            let mut movement_tmu = 0.0;
            match to_hand_info.variability() {
                Rating::High => {
                    if motion_distance < 0.0254 {
                        movement_tmu += 2.0
                    } else if 0.0254 <= motion_distance && motion_distance <= 0.1016 {
                        movement_tmu += 3.6866 * motion_distance.powf(0.6146)
                    } else if 0.1016 <= motion_distance && motion_distance <= 0.762 {
                        movement_tmu += 5.959169 + 0.690797 * motion_distance
                    } else {
                        movement_tmu +=
                            5.959169 + 0.690797 * motion_distance + 0.7 * (motion_distance - 0.762)
                    }
                }
                Rating::Medium => {
                    if motion_distance < 0.0254 {
                        movement_tmu += 2.0
                    } else if 0.0254 <= motion_distance && motion_distance <= 0.1016 {
                        movement_tmu += 2.5 * motion_distance.powf(0.681)
                    } else if 0.1016 <= motion_distance && motion_distance <= 0.762 {
                        movement_tmu += 4.309488 + 0.71666 * motion_distance
                    } else {
                        movement_tmu +=
                            4.309488 + 0.71666 * motion_distance + 0.7 * (motion_distance - 0.762)
                    }
                }
                Rating::Low => {
                    if motion_distance < 0.0254 {
                        movement_tmu += 2.0
                    } else if 0.0254 <= motion_distance && motion_distance <= 0.0762 {
                        movement_tmu += 2.5 * motion_distance.powf(0.681)
                    } else if 0.0762 <= motion_distance && motion_distance <= 0.762 {
                        movement_tmu += 4.333601 + 0.440266 * motion_distance
                    } else {
                        movement_tmu +=
                            4.333601 + 0.440266 * motion_distance + 0.4 * (motion_distance - 0.762)
                    }
                }
            }

            // Depending on the weight of the object, add a tmu penalty
            let weight = target_info.weight();
            for (lower, upper, factor, constant) in HUMAN_WEIGHT_FACTORS_MAPPING {
                if lower < weight && weight <= upper {
                    movement_tmu += factor * weight + constant;
                    break;
                }
            }
            tmu += movement_tmu;

            if get_is_within_neutral_reach(
                standing_info,
                to_hand_info,
                agent.acromial_height,
                agent.reach,
            ) {
                tmu += 30.5;
            }

            // Release tmu
            tmu += 2.0;

            let time = tmu * TMU_PER_SECOND;
            return time;
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
            // Consider Travel Calculation

            // Retrieve data
            let from_standing_info = job.points_of_interest.get(from_standing).unwrap();
            let to_standing_info = job.points_of_interest.get(to_standing).unwrap();

            // Compute travel vector/distance
            let travel_vector = to_standing_info.position() - from_standing_info.position();
            let travel_distance = travel_vector.norm();

            // Compute carry time
            return (15.0 * (travel_distance / DISTANCE_PER_PACE)) * TMU_PER_SECOND;
        }
        (
            1,
            Some(Primitive::Reach {
                from_hand, to_hand, ..
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
                        3.6866 * 39.37 * motion_distance.powf(0.6146) * TMU_PER_SECOND
                    } else if 0.1016 <= motion_distance && motion_distance <= 0.762 {
                        (5.959169 + 0.690797 * 39.37 * motion_distance) * TMU_PER_SECOND
                    } else {
                        (5.959169
                            + 0.690797 * 39.37 * motion_distance
                            + 0.7 * (39.37 * motion_distance - 0.762))
                            * TMU_PER_SECOND
                    }
                }
                Rating::Medium => {
                    if motion_distance < 0.0254 {
                        2.0 * TMU_PER_SECOND
                    } else if 0.0254 <= motion_distance && motion_distance <= 0.1016 {
                        2.5 * 39.37 * motion_distance.powf(0.681) * TMU_PER_SECOND
                    } else if 0.1016 <= motion_distance && motion_distance <= 0.762 {
                        (4.309488 + 0.71666 * 39.37 * motion_distance) * TMU_PER_SECOND
                    } else {
                        (4.309488
                            + 0.71666 * 38.37 * motion_distance
                            + 0.7 * (39.37 * motion_distance - 0.762))
                            * TMU_PER_SECOND
                    }
                }
                Rating::Low => {
                    if motion_distance < 0.0254 {
                        2.0 * TMU_PER_SECOND
                    } else if 0.0254 <= motion_distance && motion_distance <= 0.0762 {
                        2.5 * 39.37 * motion_distance.powf(0.681) * TMU_PER_SECOND
                    } else if 0.0762 <= motion_distance && motion_distance <= 0.762 {
                        (4.333601 + 0.440266 * 39.37 * motion_distance) * TMU_PER_SECOND
                    } else {
                        (4.333601
                            + 0.440266 * 39.37 * motion_distance
                            + 0.4 * (39.37 * motion_distance - 0.762))
                            * TMU_PER_SECOND
                    }
                }
            };
        }
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

            // grasp TMU
            tmu += 2.0;

            let (mvc, _hand_to_floor_dist, _dist, _is_one_hand) =
                get_force_mvc(transition, magnitude, agent, job, weight);

            if mvc < 0.2 {
                if weight < 8.9271 {
                    tmu += 4.0;
                } else {
                    tmu += 5.7;
                }
            } else if mvc < 0.5 {
                if weight < 8.9271 {
                    tmu += 7.5;
                } else {
                    tmu += 11.8;
                }
            } else {
                if weight < 8.9271 {
                    tmu += 22.9;
                } else {
                    tmu += 34.7;
                }
            }

            // release TMU
            tmu += 2.0;

            // Convert TMU to seconds
            let time = tmu * TMU_PER_SECOND;

            return time;
        }
        (
            1,
            Some(Primitive::Position {
                id,
                target,
                degrees,
                ..
            }),
        ) => {
            // Consider Force Calculation
            let mut tmu: f64 = 0.0;

            let target_info = job.targets.get(target).unwrap();
            let weight = target_info.weight();

            // Grasp tmu
            tmu += 2.0;

            // upper bounding by 360 degrees
            if weight < 8.9271 {
                tmu += 1.4927 + 0.043878 * degrees;
            } else if weight < 44.5374 {
                tmu += 2.3463636 + 0.0689090 * degrees;
            } else {
                tmu += 4.4781818 + 0.131636 * degrees;
            }

            // Release tmu
            tmu += 2.0;

            // Convert TMU to seconds
            let time = tmu * TMU_PER_SECOND;

            return time;
        }
        (
            1,
            Some(Primitive::Inspect {
                id, target, skill, ..
            }),
        ) => {
            // this primitive's time will be based on the time of the primitives coupled with it
            0.0
        }
        (1, Some(Primitive::Selection { target, .. })) => {
            let target_object = job.targets.get(target).unwrap();
            let target_size = target_object.size();

            if target_size < 0.00635 {
                return 12.9 * TMU_PER_SECOND;
            } else if target_size < 0.0254 {
                return 9.1 * TMU_PER_SECOND;
            }
            return 7.3 * TMU_PER_SECOND;
        }
        (2, Some(Primitive::Position { id, target, .. })) => {
            return match assigned_primitives.last() {
                Some(Primitive::Force {
                    id,
                    target,
                    magnitude,
                }) => {
                    let target_info = job.targets.get(target).unwrap();
                    let weight = target_info.weight();
                    let mut tmu = 0.0;

                    // Grasp TMU
                    tmu += 2.0;

                    let (mvc, _hand_to_floor_dist, _dist, _is_one_hand) =
                        get_force_mvc(transition, magnitude, agent, job, weight);

                    if mvc < 0.2 {
                        match target_info.symmetry() {
                            Rating::High => {
                                if weight < 8.9271 {
                                    tmu += 5.6;
                                } else {
                                    tmu += 11.2;
                                }
                            }
                            Rating::Medium => {
                                if weight < 8.9271 {
                                    tmu += 9.1;
                                } else {
                                    tmu += 14.7;
                                }
                            }
                            Rating::Low => {
                                if weight < 8.9271 {
                                    tmu += 10.4;
                                } else {
                                    tmu += 16.0;
                                }
                            }
                        }
                    } else if mvc < 0.5 {
                        match target_info.symmetry() {
                            Rating::High => {
                                if weight < 8.9271 {
                                    tmu += 16.2;
                                } else {
                                    tmu += 21.8;
                                }
                            }
                            Rating::Medium => {
                                if weight < 8.9271 {
                                    tmu += 19.7;
                                } else {
                                    tmu += 25.3;
                                }
                            }
                            Rating::Low => {
                                if weight < 8.9271 {
                                    tmu += 21.0;
                                } else {
                                    tmu += 26.6;
                                }
                            }
                        }
                    } else {
                        match target_info.symmetry() {
                            Rating::High => {
                                if weight < 8.9271 {
                                    tmu += 43.0;
                                } else {
                                    tmu += 48.6;
                                }
                            }
                            Rating::Medium => {
                                if weight < 8.9271 {
                                    tmu += 46.5;
                                } else {
                                    tmu += 52.1;
                                }
                            }
                            Rating::Low => {
                                if weight < 8.9271 {
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
                }
                _ => 0.0,
            };
        }
        (
            2,
            Some(Primitive::Force {
                id,
                target,
                magnitude,
                ..
            }),
        ) => {
            return match assigned_primitives.last() {
                Some(Primitive::Position {
                    id,
                    target,
                    degrees,
                }) => {
                    let target_info = job.targets.get(target).unwrap();
                    let weight = target_info.weight();
                    let mut tmu = 0.0;

                    // Grasp TMU
                    tmu += 2.0;

                    let (mvc, _hand_to_floor_dist, _dist, _is_one_hand) =
                        get_force_mvc(transition, magnitude, agent, job, weight);

                    if mvc < 0.2 {
                        match target_info.symmetry() {
                            Rating::High => {
                                if weight < 8.9271 {
                                    tmu += 5.6;
                                } else {
                                    tmu += 11.2;
                                }
                            }
                            Rating::Medium => {
                                if weight < 8.9271 {
                                    tmu += 9.1;
                                } else {
                                    tmu += 14.7;
                                }
                            }
                            Rating::Low => {
                                if weight < 8.9271 {
                                    tmu += 10.4;
                                } else {
                                    tmu += 16.0;
                                }
                            }
                        }
                    } else if mvc < 0.5 {
                        match target_info.symmetry() {
                            Rating::High => {
                                if weight < 8.9271 {
                                    tmu += 16.2;
                                } else {
                                    tmu += 21.8;
                                }
                            }
                            Rating::Medium => {
                                if weight < 8.9271 {
                                    tmu += 19.7;
                                } else {
                                    tmu += 25.3;
                                }
                            }
                            Rating::Low => {
                                if weight < 8.9271 {
                                    tmu += 21.0;
                                } else {
                                    tmu += 26.6;
                                }
                            }
                        }
                    } else {
                        match target_info.symmetry() {
                            Rating::High => {
                                if weight < 8.9271 {
                                    tmu += 43.0;
                                } else {
                                    tmu += 48.6;
                                }
                            }
                            Rating::Medium => {
                                if weight < 8.9271 {
                                    tmu += 46.5;
                                } else {
                                    tmu += 52.1;
                                }
                            }
                            Rating::Low => {
                                if weight < 8.9271 {
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
                }
                _ => 0.0,
            };
        }
        (_, _) => {
            // There is some non-zero number of assigned primitives. Compute them independently and run the max on them
            0.0
        }
    };
}

fn get_robot_time_for_primitive(
    assigned_primitives: Vec<&Primitive>,
    transition: &Transition,
    job: &Job,
    agent: &RobotInfo,
) -> Time {
    return match (assigned_primitives.len(), assigned_primitives.first()) {
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
            // Calculate mobile base travel distance
            let from_standing_info = job.points_of_interest.get(from_standing).unwrap();
            let to_standing_info = job.points_of_interest.get(to_standing).unwrap();
            let standing_vector = from_standing_info.position() - to_standing_info.position();
            let standing_distance = standing_vector.norm();
            let standing_travel_time = standing_distance / agent.mobile_speed;

            // Calculate manipulator travel distance
            let from_hand_info = job.points_of_interest.get(from_hand).unwrap();
            let to_hand_info = job.points_of_interest.get(to_hand).unwrap();
            let hand_vector = from_hand_info.position() - to_hand_info.position();
            let hand_distance = hand_vector.norm();
            let hand_travel_time = hand_distance / agent.speed;

            // TODO: should this be max or additive?
            return standing_travel_time + hand_travel_time;
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
            // Calculate manipulator travel distance
            let from_hand_info = job.points_of_interest.get(from_hand).unwrap();
            let to_hand_info = job.points_of_interest.get(to_hand).unwrap();
            let hand_vector = from_hand_info.position() - to_hand_info.position();
            let hand_distance = hand_vector.norm();
            let hand_travel_time = hand_distance / agent.speed;
            return hand_travel_time;
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
            // Calculate mobile base travel distance
            let from_standing_info = job.points_of_interest.get(from_standing).unwrap();
            let to_standing_info = job.points_of_interest.get(to_standing).unwrap();
            let standing_vector = from_standing_info.position() - to_standing_info.position();
            let standing_distance = standing_vector.norm();
            let standing_travel_time = standing_distance / agent.mobile_speed;
            return standing_travel_time;
        }
        (
            1,
            Some(Primitive::Reach {
                from_hand, to_hand, ..
            }),
        ) => {
            // Calculate manipulator travel distance
            let from_hand_info = job.points_of_interest.get(from_hand).unwrap();
            let to_hand_info = job.points_of_interest.get(to_hand).unwrap();
            let hand_vector = from_hand_info.position() - to_hand_info.position();
            let hand_distance = hand_vector.norm();
            let hand_travel_time = hand_distance / agent.speed;
            return hand_travel_time;
        }
        (
            1,
            Some(Primitive::Force {
                id,
                target,
                magnitude,
                ..
            }),
        ) => {
            // TODO
            let mut time_delta = 0.0;

            // grasp time (based on precision)
            time_delta += 5.0;

            // duration of force
            time_delta += 1.0;

            // release time
            time_delta += 1.0;

            return time_delta;
        }
        (
            1,
            Some(Primitive::Position {
                id,
                target,
                degrees,
                ..
            }),
        ) => {
            // TODO: fix this
            let mut time_delta = 0.0;

            // grasp time (based on precision)
            time_delta += 5.0;

            // position time (by max speed)
            time_delta += 0.5 + degrees / agent.speed;

            // release
            time_delta += 1.0;

            return time_delta;
        }
        (
            1,
            Some(Primitive::Inspect {
                id, target, skill, ..
            }),
        ) => {
            // this primitive's time will be based on sensor rating
            if agent.sensing == Rating::Low {
                return 3.0;
            } else if agent.sensing == Rating::Medium {
                return 1.0;
            } else {
                return 0.5;
            }
        }
        (
            1,
            Some(Primitive::Selection {
                id, target, skill, ..
            }),
        ) => {
            // TODO: have some base time for sensing and then time for movement

            // this primitive's time will be based on sensor rating
            if agent.sensing == Rating::Low {
                return 10.0;
            } else if agent.sensing == Rating::Medium {
                return 5.0;
            } else {
                return 2.5;
            }
        }
        (
            2,
            Some(Primitive::Position {
                id,
                target,
                degrees,
            }),
        ) => {
            return match assigned_primitives.last() {
                Some(Primitive::Force {
                    id,
                    target,
                    magnitude,
                }) => {
                    // TODO: based on magnitude of force and symmetry of object?????

                    // TODO: fix this
                    let mut time_delta = 0.0;

                    // grasp time (based on precision)
                    time_delta += 5.0;

                    // position time (by max speed)
                    time_delta += 0.5 + degrees / agent.speed;

                    // release
                    time_delta += 1.0;

                    return time_delta;
                }
                _ => 0.0,
            };
        }
        (
            2,
            Some(Primitive::Force {
                id,
                target,
                magnitude,
                ..
            }),
        ) => {
            return match assigned_primitives.last() {
                Some(Primitive::Position {
                    id,
                    target,
                    degrees,
                }) => {
                    // TODO: based on magnitude of force and symmetry of object?????

                    // TODO: fix this
                    let mut time_delta = 0.0;

                    // grasp time (based on precision)
                    time_delta += 5.0;

                    // position time (by max speed)
                    time_delta += 0.5 + degrees / agent.speed;

                    // release
                    time_delta += 1.0;

                    return time_delta;
                }
                _ => 0.0,
            };
        }
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
