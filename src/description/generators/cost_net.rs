use crate::description::job::Job;
use crate::description::agent::{Agent, CostProfiler};
use crate::petri::data::{Data, DataTag, Query};
use crate::petri::net::PetriNet;
use crate::petri::place::Place;
use crate::petri::token::TokenSet;
use crate::petri::transition::{Signature, Transition};
use enum_tag::EnumTag;
use std::cmp;
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

                    // For each ergo type, create a place for those tokens to go.
                    let ergo_cost_data = vec![
                        Data::ErgoWholeBody(*id, 0),
                        Data::ErgoArm(*id, 0),
                        Data::ErgoHand(*id, 0),
                    ];

                    let ergo_cost_places: Vec<Place> = ergo_cost_data
                        .iter()
                        .map(|data| {
                            Place::new(
                                format!("{} {:?}", human.name, data.tag()),
                                TokenSet::Finite,
                                vec![data.clone()],
                            )
                        })
                        .collect();

                    // Add them and set their initial marking values to 0
                    for place in ergo_cost_places.iter() {
                        net.places.insert(place.id, place.clone());
                        net.initial_marking.insert(place.id, 0);
                    }

                    let ergo_cost_place_ids: Vec<Uuid> =
                        ergo_cost_places.iter().map(|p| p.id).collect();

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
                        for (ergo_idx, ergo_type) in ergo_cost_data.iter().enumerate() {
                            let recovery: usize = match ergo_type.tag() {
                                DataTag::ErgoWholeBody => {
                                    human.ergo_recovery_whole(&transition, &self)
                                }
                                DataTag::ErgoArm => human.ergo_recovery_arm(&transition, &self),
                                DataTag::ErgoHand => human.ergo_recovery_hand(&transition, &self),
                                _ => 0,
                            };
                            let cost: usize = match ergo_type.tag() {
                                DataTag::ErgoWholeBody => human.ergo_cost_whole(&transition, &self),
                                DataTag::ErgoArm => human.ergo_cost_arm(&transition, &self),
                                DataTag::ErgoHand => human.ergo_cost_hand(&transition, &self),
                                _ => 0,
                            };

                            if recovery > 0 {
                                transition_copy.input.insert(
                                    ergo_cost_place_ids[ergo_idx],
                                    Signature::Range(0, recovery),
                                );
                            }

                            if cost > 0 {
                                transition_copy
                                    .output
                                    .insert(ergo_cost_place_ids[ergo_idx], Signature::Static(1));
                            }
                        }

                        let onetime_cost: usize = human.onetime_cost(&transition, &self);
                        let execution_time: usize = human.execution_time(&transition, &self);

                        transition_copy.time = cmp::max(transition_copy.time, execution_time);
                        transition_copy.cost += onetime_cost;

                        updated_transitions.insert(transition.id, transition_copy);
                    }

                    // Update the transitions with the new versions
                    for (id, transition) in updated_transitions {
                        net.transitions.insert(id, transition);
                    }
                }
                _ => { /* No Ergo calculations for robots */ }
            }
            // Add a place for each ergo bin
        }

        println!(
            "Cost Net: Places {:?}, Transitions {:?}",
            net.places.len(),
            net.transitions.len()
        );

        net
    }
}