use crate::description::job::Job;
use crate::description::agent::{Agent, CostProfiler};
use crate::description::rating::Rating;
use crate::description::units::Time;
use crate::petri::cost::{add_cost_sets, Cost, CostSet, CostCategory, CostFrequency};
use crate::petri::data::{Data, DataTag, Query};
use crate::petri::net::PetriNet;
use crate::petri::place::Place;
use crate::petri::token::TokenSet;
use crate::petri::transition::{self, Signature, Transition};
use enum_tag::EnumTag;
use itertools::Itertools;
use std::collections::HashMap;
use uuid::Uuid;

impl Job {
    pub fn compute_cost_from_poi(&self) -> PetriNet {
        let poi_net = self.poi_net.as_ref().unwrap();
        let mut net = poi_net.clone();
        for (id, agent) in self.agents.iter() {
            match agent {
                Agent::Human(human) => {
                    // Create a new place to store updated transitions.
                    // This will be merged into the other transitions at the end.
                    let mut updated_transitions: HashMap<Uuid, Transition> = HashMap::new();
                    let mut remove_transitions: Vec<Uuid> = Vec::new();

                    // For each ergo type, create a place for those tokens to go.
                    let ergo_cost_data = vec![
                        Data::ErgoWholeBody(*id, 0.0),
                        Data::ErgoArm(*id, 0.0),
                        Data::ErgoHand(*id, 0.0),
                    ];

                    // let ergo_cost_places: Vec<Place> = ergo_cost_data
                    //     .iter()
                    //     .map(|data| {
                    //         Place::new(
                    //             format!("{} {:?}", human.name, data.tag()),
                    //             TokenSet::Finite,
                    //             vec![data.clone()],
                    //         )
                    //     })
                    //     .collect();

                    // Add them and set their initial marking values to 0
                    // for place in ergo_cost_places.iter() {
                    //     net.places.insert(place.id, place.clone());
                    //     net.initial_marking.insert(place.id, 0);
                    // }

                    // let ergo_cost_place_ids: Vec<Uuid> =
                    //     ergo_cost_places.iter().map(|p| p.id).collect();

                    // Find all action-based transitions that are relevant to this agent
                    for transition in net.query_transitions(&vec![
                        Query::Data(Data::Agent(*id)),
                        Query::Data(Data::Action(*id)),
                    ]) {
                        // This should return only transitions representing actual actions
                        // These include actions specified by the user, but also ones we added (e.g. travel, reach, etc)

                        // If we already updated this transition, use the updated version
                        let mut transition_copy = match updated_transitions.get(&transition.id) {
                            Some(t) => t.clone(),
                            None => transition.clone(),
                        };

                        // let meta_data = transition.meta_data.clone();
                        // for (ergo_idx, ergo_type) in ergo_cost_data.iter().enumerate() {
                        //     let recovery: usize = match ergo_type.tag() {
                        //         DataTag::ErgoWholeBody => {
                        //             human.ergo_recovery_whole(&transition, &self)
                        //         }
                        //         DataTag::ErgoArm => human.ergo_recovery_arm(&transition, &self),
                        //         DataTag::ErgoHand => human.ergo_recovery_hand(&transition, &self),
                        //         _ => 0,
                        //     };
                        //     let cost: usize = match ergo_type.tag() {
                        //         DataTag::ErgoWholeBody => human.ergo_cost_whole(&transition, &self),
                        //         DataTag::ErgoArm => human.ergo_cost_arm(&transition, &self),
                        //         DataTag::ErgoHand => human.ergo_cost_hand(&transition, &self),
                        //         _ => 0,
                        //     };

                        //     if recovery > 0 {
                        //         transition_copy.input.insert(
                        //             ergo_cost_place_ids[ergo_idx],
                        //             Signature::Range(0, recovery),
                        //         );
                        //     }

                        //     if cost > 0 {
                        //         transition_copy
                        //             .output
                        //             .insert(ergo_cost_place_ids[ergo_idx], Signature::Static(1));
                        //     }
                        // }

                        let (cost_set, new_ergo_meta_data): (CostSet, Vec<Data>) = human.cost_set(&transition, &self);

                        for ergo_meta_data in new_ergo_meta_data.iter() {
                            // Add the new meta data flags to the copy of the transition
                            transition_copy.add_data(ergo_meta_data.clone());

                            // If transition has MVC >= 150%, remove it.
                            match ergo_meta_data {
                                Data::MVC(_, n) => {
                                    if *n > 1.0 {
                                        remove_transitions.push(transition.id);
                                    }
                                }
                                _ => {}
                            }
                        }

                        let execution_time: Time = human.execution_time(&transition, &self);

                        let time: Time;
                        // if transition_copy.time < execution_time {
                        //     time = transition_copy.time;
                        // } else {
                            time = execution_time;
                        // }
                        transition_copy.time = time;
                        transition_copy.cost = add_cost_sets(&transition.cost, &cost_set);

                        updated_transitions.insert(transition.id, transition_copy);
                        
                    }

                    // Compute costs for spawned or produced parts
                    for transition in net.query_transitions_any(&vec![
                        Query::Tag(DataTag::Spawn),
                        Query::Tag(DataTag::Produce),
                        Query::Data(Data::AgentAdd(*id))
                    ]) {
                        // If we already updated this transition, use the updated version
                        let mut transition_copy = match updated_transitions.get(&transition.id) {
                            Some(t) => t.clone(),
                            None => transition.clone(),
                        };

                        let (cost_set, _new_ergo_meta_data): (CostSet, Vec<Data>) = human.cost_set(&transition, &self);

                        transition_copy.cost = add_cost_sets(&transition.cost, &cost_set);

                        updated_transitions.insert(transition.id, transition_copy);
                    }

                    // Update the transitions with the new versions
                    for (id, transition) in updated_transitions {
                        net.transitions.insert(id, transition);
                    }

                    for id in remove_transitions {
                        net.delete_transition(id);
                    }
                }
                Agent::Robot(robot) => {
                    // Create a new place to store updated transitions.
                    // This will be merged into the other transitions at the end.
                    let mut updated_transitions: HashMap<Uuid, Transition> = HashMap::new();
                    let mut remove_transitions: Vec<Uuid> = Vec::new();

                    // Find all transitions that are relevant to this agent
                    for transition in net.query_transitions(&vec![
                        Query::Data(Data::Agent(*id)),
                        Query::Data(Data::Action(*id)),
                    ]) {
                        // If we already updated this transition, use the updated version
                        let mut transition_copy = match updated_transitions.get(&transition.id) {
                            Some(t) => t.clone(),
                            None => transition.clone(),
                        };

                        let (cost_set, new_ergo_meta_data): (CostSet, Vec<Data>) = robot.cost_set(&transition, &self);
                        let execution_time: Time = robot.execution_time(&transition, &self);

                        
                        for ergo_meta_data in new_ergo_meta_data.iter() {
                            // If transition has MVC, remove it.
                            match ergo_meta_data {
                                Data::MVC(_, _n) => {
                                    remove_transitions.push(transition.id);
                                }
                                _ => {}
                            }
                        }

                        let time: Time;
                        // if transition_copy.time < execution_time {
                        //     time = transition_copy.time;
                        // } else {
                            time = execution_time;
                        // }
                        transition_copy.time = time;
                        transition_copy.cost = add_cost_sets(&transition.cost, &cost_set);

                        updated_transitions.insert(transition.id, transition_copy);
                    }


                    for transition in net.query_transitions(&vec![
                        Query::Data(Data::AgentAdd(*id)),
                    ]) {
                        // If we already updated this transition, use the updated version
                        let mut transition_copy = match updated_transitions.get(&transition.id) {
                            Some(t) => t.clone(),
                            None => transition.clone(),
                        };

                        let mut cost_set: Vec<Cost> = CostSet::new();
                        cost_set.push(Cost {
                            frequency: CostFrequency::Once,
                            value: robot.purchase_price,
                            category: CostCategory::Monetary,
                        });
                        
                        // Cost for integration (4-6x)
                        // https://www.engineering.com/time-and-money-how-much-do-industrial-robots-cost/
                        // TODO: update the value based on needed systems (i.e. integration breakdown - vision, pick+place, etc))
                        let multiplicative_factor = if robot.sensing == Rating::High { 6.0 } else if robot.sensing == Rating::Medium { 5.0 } else { 4.0 }; 
                        cost_set.push(Cost {
                            frequency: CostFrequency::Once,
                            value: robot.purchase_price * multiplicative_factor,
                            category: CostCategory::Monetary,
                        });

                        transition_copy.cost = add_cost_sets(&transition.cost, &cost_set);
                        updated_transitions.insert(transition.id, transition_copy);
                    }


                    // Update the transitions with the new versions
                    for (id, transition) in updated_transitions {
                        net.transitions.insert(id, transition);
                    }

                    for id in remove_transitions {
                        net.delete_transition(id);
                    }
                }
            }
            // Add a place for each ergo bin
        }

        // Check allocations and make sure they have outgoing arcs, if not we can prune the relevant decide action
        let mut remove_transitions: Vec<Uuid> = Vec::new();
        for transition in net.query_transitions(&vec![Query::Data(Data::Decide)]) {
            for key in transition.output.keys() {
                let mut keep_allocation = false;

                // if key is not in the input set, then it's the allocation location
                if !transition.input.keys().contains(key) {
                    for some_transition in net.transitions.keys() {
                        if *some_transition != transition.id && net.transitions.get(some_transition).unwrap().input.contains_key(key) {
                            keep_allocation = true;
                        }
                    }

                    // remove the transition, as this agent/combo can't perform the action
                    if !keep_allocation {
                        remove_transitions.push(transition.id);
                    }
                }
            }
        }

        for id in remove_transitions {
            net.delete_transition(id);
        }

        println!(
            "Cost Net: Places {:?}, Transitions {:?}",
            net.places.len(),
            net.transitions.len()
        );

        net
    }
}