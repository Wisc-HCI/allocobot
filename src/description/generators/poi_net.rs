use crate::description::job::Job;
use crate::description::poi::PointOfInterest;
use crate::description::primitive::Primitive;
use crate::petri::data::{Data, DataTag, Query};
use crate::petri::net::PetriNet;
use crate::petri::place::Place;
use crate::petri::transition::{self, Signature, Transition};
use enum_tag::EnumTag;
use itertools::Itertools;
use std::collections::HashMap;
use uuid::Uuid;

impl Job {
    pub fn compute_poi_from_agent(&mut self) -> PetriNet {
        let agent_net = self.agent_net.as_ref().unwrap();
        let mut net = agent_net.clone();
        let mut standing_pois: Vec<&PointOfInterest> = vec![];
        let mut hand_pois: Vec<&PointOfInterest> = vec![];
        let mut new_names: HashMap<Uuid, String> = HashMap::new();

        for (poi_id, poi) in self.points_of_interest.iter() {
            match poi {
                PointOfInterest::Standing(_) => standing_pois.push(poi),
                PointOfInterest::Hand(_) => hand_pois.push(poi),
            }
            new_names.insert(poi_id.clone(), poi.name().clone());
        }

        // For each agent spawn location, create a standing place
        for (agent_id, agent) in self.agents.iter() {
            // Determine the pairs of valid standing/hand poi pairs
            let valid_pairs: Vec<(&PointOfInterest, &PointOfInterest)> = standing_pois
                .iter()
                .cartesian_product(&hand_pois)
                .filter(|(standing_poi, hand_poi)| standing_poi.reachability(hand_poi, agent))
                .map(|(standing_poi, hand_poi)| (*standing_poi, *hand_poi))
                .collect();

            let mut new_transitions: Vec<Transition> = vec![];

            let agent_situated_places =
                agent_net.query_places(&vec![Query::Data(Data::AgentSituated(*agent_id))]);
            for agent_situated_place in agent_situated_places {
                net.split_place(
                    &agent_situated_place.id,
                    valid_pairs
                        .iter()
                        .map(|(standing_poi, hand_poi)| {
                            vec![
                                Data::Standing(standing_poi.id(), *agent_id),
                                Data::Hand(hand_poi.id(), *agent_id),
                            ]
                        })
                        .collect(),
                    |_transition, _split_data| true,
                );
                net.query_places(&vec![
                    Query::Tag(DataTag::Standing),
                    Query::Tag(DataTag::Hand),
                    Query::Data(Data::AgentSituated(*agent_id)),
                ])
                .iter()
                .tuple_combinations()
                .for_each(|(place1, place2)| {
                    let standing_poi_id1: Uuid = place1
                        .meta_data
                        .iter()
                        .find(|d| d.tag() == DataTag::Standing)
                        .unwrap()
                        .id()
                        .unwrap();
                    let standing_poi_id2: Uuid = place2
                        .meta_data
                        .iter()
                        .find(|d| d.tag() == DataTag::Standing)
                        .unwrap()
                        .id()
                        .unwrap();
                    let hand_poi_id1: Uuid = place1
                        .meta_data
                        .iter()
                        .find(|d| d.tag() == DataTag::Hand)
                        .unwrap()
                        .id()
                        .unwrap();
                    let hand_poi_id2: Uuid = place2
                        .meta_data
                        .iter()
                        .find(|d| d.tag() == DataTag::Hand)
                        .unwrap()
                        .id()
                        .unwrap();
                    let standing_poi1 = self.points_of_interest.get(&standing_poi_id1).unwrap();
                    let standing_poi2 = self.points_of_interest.get(&standing_poi_id2).unwrap();
                    let hand_poi1 = self.points_of_interest.get(&hand_poi_id1).unwrap();
                    let hand_poi2 = self.points_of_interest.get(&hand_poi_id2).unwrap();
                    if standing_poi1 == standing_poi2 {
                        // This is strictly a hand reach
                        if standing_poi1.reachability(hand_poi2, agent) {
                            // One Way 1->2
                            let primitive1 = Primitive::Reach {
                                id: Uuid::new_v4(),
                                standing: standing_poi_id1,
                                from_hand: hand_poi_id1,
                                to_hand: hand_poi_id2,
                            };
                            let primitive_id1 = primitive1.id();
                            new_names.insert(primitive_id1, format!("{:?}", primitive1.tag()));
                            self.primitives.insert(primitive_id1, primitive1);

                            let transition1 = Transition::new(
                                format!(
                                    "{}:Reach:{}->{}",
                                    agent.name(),
                                    hand_poi1.name(),
                                    hand_poi2.name()
                                ),
                                vec![(place1.id, Signature::Static(1))]
                                    .into_iter()
                                    .collect(),
                                vec![(place2.id, Signature::Static(1))]
                                    .into_iter()
                                    .collect(),
                                vec![
                                    Data::Simulation,
                                    Data::Agent(*agent_id),
                                    Data::Standing(standing_poi_id1, *agent_id),
                                    Data::FromHandPOI(hand_poi_id1, *agent_id),
                                    Data::ToHandPOI(hand_poi_id2, *agent_id),
                                    Data::Action(*agent_id),
                                    Data::PrimitiveAssignment(*agent_id, primitive_id1),
                                ],
                                0.0,
                                vec![],
                            );
                            new_transitions.push(transition1);

                            // Now the other way 2->1
                            let primitive2 = Primitive::Reach {
                                id: Uuid::new_v4(),
                                standing: standing_poi_id1,
                                from_hand: hand_poi_id2,
                                to_hand: hand_poi_id1,
                            };
                            let primitive_id2 = primitive2.id();
                            new_names.insert(primitive_id2, format!("{:?}", primitive2.tag()));
                            self.primitives.insert(primitive_id2, primitive2);

                            let transition2 = Transition::new(
                                format!(
                                    "{}:Reach:{}->{}",
                                    agent.name(),
                                    hand_poi2.name(),
                                    hand_poi1.name()
                                ),
                                vec![(place2.id, Signature::Static(1))]
                                    .into_iter()
                                    .collect(),
                                vec![(place1.id, Signature::Static(1))]
                                    .into_iter()
                                    .collect(),
                                vec![
                                    Data::Simulation,
                                    Data::Agent(*agent_id),
                                    Data::Standing(standing_poi_id1, *agent_id),
                                    Data::FromHandPOI(hand_poi_id2, *agent_id),
                                    Data::ToHandPOI(hand_poi_id1, *agent_id),
                                    Data::Action(*agent_id),
                                    Data::PrimitiveAssignment(*agent_id, primitive_id2),
                                ],
                                0.0,
                                vec![],
                            );
                            new_transitions.push(transition2);
                        }
                    } else if standing_poi1.travelability(standing_poi2, agent) {
                        // This is is a travel (plus reach if the hand poi is different)

                        // One Way 1->2
                        let primitive1 = Primitive::Travel {
                            id: Uuid::new_v4(),
                            from_standing: standing_poi_id1,
                            to_standing: standing_poi_id2,
                            from_hand: hand_poi_id1,
                            to_hand: hand_poi_id2,
                        };

                        let primitive_id1 = primitive1.id();
                        new_names.insert(primitive_id1, format!("{:?}", primitive1.tag()));
                        self.primitives.insert(primitive_id1, primitive1);

                        let primitive2 = Primitive::Travel {
                            id: Uuid::new_v4(),
                            from_standing: standing_poi_id2,
                            to_standing: standing_poi_id1,
                            from_hand: hand_poi_id2,
                            to_hand: hand_poi_id1,
                        };

                        let primitive_id2 = primitive2.id();
                        new_names.insert(primitive_id2, format!("{:?}", primitive2.tag()));
                        self.primitives.insert(primitive_id2, primitive2);

                        let transition1 = Transition::new(
                            format!(
                                "{}:Travel:{}->{}",
                                agent.name(),
                                standing_poi1.name(),
                                standing_poi2.name()
                            ),
                            vec![(place1.id, Signature::Static(1))]
                                .into_iter()
                                .collect(),
                            vec![(place2.id, Signature::Static(1))]
                                .into_iter()
                                .collect(),
                            vec![
                                Data::Simulation,
                                Data::Agent(*agent_id),
                                Data::Hand(hand_poi_id1, *agent_id),
                                Data::FromStandingPOI(standing_poi_id1, *agent_id),
                                Data::ToStandingPOI(standing_poi_id2, *agent_id),
                                Data::FromHandPOI(hand_poi_id1, *agent_id),
                                Data::ToHandPOI(hand_poi_id2, *agent_id),
                                Data::Action(*agent_id),
                                Data::PrimitiveAssignment(*agent_id, primitive_id1),
                            ],
                            0.0,
                            vec![],
                        );
                        new_transitions.push(transition1);
                        let transition2 = Transition::new(
                            format!(
                                "{}:Travel:{}->{}",
                                agent.name(),
                                standing_poi2.name(),
                                standing_poi1.name()
                            ),
                            vec![(place2.id, Signature::Static(1))]
                                .into_iter()
                                .collect(),
                            vec![(place1.id, Signature::Static(1))]
                                .into_iter()
                                .collect(),
                            vec![
                                Data::Simulation,
                                Data::Agent(*agent_id),
                                Data::Hand(hand_poi_id1, *agent_id),
                                Data::FromStandingPOI(standing_poi_id2, *agent_id),
                                Data::ToStandingPOI(standing_poi_id1, *agent_id),
                                Data::FromHandPOI(hand_poi_id2, *agent_id),
                                Data::ToHandPOI(hand_poi_id1, *agent_id),
                                Data::Action(*agent_id),
                                Data::PrimitiveAssignment(*agent_id, primitive_id2),
                            ],
                            0.0,
                            vec![],
                        );
                        new_transitions.push(transition2);
                    }
                });
            }

            for transition in new_transitions {
                net.transitions.insert(transition.id, transition);
            }
        }

        // Add a no-op (rest) transition for each agent situated place
        let mut new_rest_transitions = vec![];
        for (agent_id, agent) in self.agents.iter() {
            let agent_situated_places =
                net.query_places(&vec![Query::Data(Data::AgentSituated(*agent_id))]);
            // println!(
            //     "Agent {} has {} situated places",
            //     agent.name(),
            //     agent_situated_places.len()
            // );
            for place in agent_situated_places.iter() {
                let standing_poi_data = place
                    .meta_data
                    .iter()
                    .find(|d| d.tag() == DataTag::Standing);
                let hand_poi_data = place.meta_data.iter().find(|d| d.tag() == DataTag::Hand);
                match (standing_poi_data, hand_poi_data) {
                    (
                        Some(Data::Standing(standing_poi_id, standing_agent_id)),
                        Some(Data::Hand(hand_poi_id, hand_agent_id)),
                    ) => {
                        let transition = Transition::new(
                            format!("{}:Rest", agent.name()),
                            vec![(place.id, Signature::Static(1))].into_iter().collect(),
                            vec![(place.id, Signature::Static(1))].into_iter().collect(),
                            vec![
                                Data::Simulation,
                                Data::Agent(*agent_id),
                                Data::Standing(*standing_poi_id, *standing_agent_id),
                                Data::Hand(*hand_poi_id, *hand_agent_id),
                                Data::Rest(*agent_id),
                            ],
                            0.1,
                            vec![],
                        );
                        // println!("Rest Transition: {:?}", transition);
                        new_rest_transitions.push(transition);
                    }
                    _ => {}
                }
            }
        }
        new_rest_transitions.iter().for_each(|transition| {
            net.transitions.insert(transition.id, transition.clone());
        });

        // Refine the task transitions to include only those POIs defined in the task (if applicable)
        net.transitions.retain(|_, transition| {
            if transition
                .meta_data
                .iter()
                .any(|d| d.tag() == DataTag::Task)
                && transition
                    .meta_data
                    .iter()
                    .any(|d| d.tag() == DataTag::Hand)
            {
                let task_id = transition
                    .meta_data
                    .iter()
                    .find(|d| d.tag() == DataTag::Task)
                    .unwrap()
                    .id()
                    .unwrap();
                let task = self.tasks.get(&task_id).unwrap();
                // println!("Transition Meta Data: {:?}", transition.meta_data);
                let transition_hand_pois: Vec<Uuid> = transition
                    .meta_data
                    .iter()
                    .filter(|d| d.tag() == DataTag::Hand)
                    .map(|d| d.id().unwrap())
                    .collect::<Vec<Uuid>>();

                let transition_hand_poi = transition_hand_pois.first().unwrap();

                if !task.pois.is_empty() && !task.pois.contains(&transition_hand_poi) {
                    return false;
                }

                if transition_hand_pois
                    .into_iter()
                    .unique()
                    .collect::<Vec<Uuid>>()
                    .len()
                    > 1
                {
                    // println!("Transition Hand POIs: {:#?}", transition.meta_data);
                    return false;
                }
            }
            true
        });

        let mut new_transitions: Vec<Transition> = vec![];
        for (target_id, target) in self.targets.iter() {
            // Find the current target situated place. There should be only one, so query for it.
            let target_place_id = net
                .places
                .values()
                .find(|place| {
                    place
                        .meta_data
                        .iter()
                        .any(|d| *d == Data::TargetSituated(*target_id))
                })
                .unwrap()
                .id;

            // Split that node by all the valid hand locations.
            let (new_places, _) = net.split_place(
                &target_place_id,
                hand_pois
                    .iter()
                    // .filter(|hpoi| target.pois().is_empty() || target.pois().contains(&hpoi.id()))
                    .map(|hand_poi| vec![Data::Hand(hand_poi.id(), *target_id)])
                    .collect::<Vec<Vec<Data>>>(),
                |transition, split_data| {
                    if transition.has_any_data(&vec![Query::Data(Data::TargetSituated(*target_id)), Query::Data(Data::TargetLocationSelected(*target_id))]) {
                        return true;
                    }
                    let hand_poi_id = split_data
                        .iter()
                        .find(|d| d.tag() == DataTag::Hand)
                        .unwrap()
                        .id()
                        .unwrap();
                    return transition
                        .has_data(&vec![Query::PartialTagPrimary(DataTag::Hand, hand_poi_id)]);
                },
            );

            new_places
                .iter()
                .tuple_combinations()
                .for_each(|(place1_id, place2_id)| {
                    // println!("Place1: {:?}, Place2: {:?}", place1_id, place2_id);
                    let place1 = net.places.get(place1_id).unwrap();
                    let place2 = net.places.get(place2_id).unwrap();
                    let hand_id_1 = place1
                        .meta_data
                        .iter()
                        .find(|d| d.tag() == DataTag::Hand)
                        .unwrap()
                        .id()
                        .unwrap();
                    let hand_id_2 = place2
                        .meta_data
                        .iter()
                        .find(|d| d.tag() == DataTag::Hand)
                        .unwrap()
                        .id()
                        .unwrap();
                    let hand_poi1 = self.points_of_interest.get(&hand_id_1).unwrap();
                    let hand_poi2 = self.points_of_interest.get(&hand_id_2).unwrap();

                    let existing_reach_transitions = net.query_transitions(&vec![
                        Query::Tag(DataTag::Agent),
                        Query::PartialTagPrimary(DataTag::FromHandPOI, hand_id_1),
                        Query::PartialTagPrimary(DataTag::ToHandPOI, hand_id_2),
                    ]);

                    for existing_reach_transition in existing_reach_transitions {
                        let agent_id = existing_reach_transition
                            .meta_data
                            .iter()
                            .find(|d| d.tag() == DataTag::Agent)
                            .unwrap()
                            .id()
                            .unwrap();
                        let standing_poi_id1 = existing_reach_transition
                            .meta_data
                            .iter()
                            .find(|d| {
                                d.tag() == DataTag::Standing || d.tag() == DataTag::FromStandingPOI
                            })
                            .unwrap()
                            .id()
                            .unwrap();
                        let standing_poi_id2 = existing_reach_transition
                            .meta_data
                            .iter()
                            .find(|d| {
                                d.tag() == DataTag::Standing || d.tag() == DataTag::ToStandingPOI
                            })
                            .unwrap()
                            .id()
                            .unwrap();
                        let agent_name = self.agents.get(&agent_id).unwrap().name();
                        let mut input1 = existing_reach_transition.input.clone();
                        let mut output1 = existing_reach_transition.output.clone();

                        let meta_data1: Vec<Data>;
                        let meta_data2: Vec<Data>;

                        let mut is_move = false;
                        if standing_poi_id1 == standing_poi_id2 {
                            is_move = true;
                            let primitive1 = Primitive::Move {
                                id: Uuid::new_v4(),
                                target: *target_id,
                                standing: standing_poi_id1,
                                from_hand: hand_id_1,
                                to_hand: hand_id_2,
                            };

                            let primitive_id1 = primitive1.id();
                            new_names.insert(primitive1.id(), format!("{:?}", primitive1.tag()));
                            self.primitives.insert(primitive_id1, primitive1);

                            let primitive2 = Primitive::Move {
                                id: Uuid::new_v4(),
                                target: *target_id,
                                standing: standing_poi_id1,
                                from_hand: hand_id_2,
                                to_hand: hand_id_1,
                            };

                            let primitive_id2 = primitive2.id();
                            new_names.insert(primitive2.id(), format!("{:?}", primitive2.tag()));
                            self.primitives.insert(primitive_id2, primitive2);

                            meta_data1 = vec![
                                Data::Simulation,
                                Data::Agent(agent_id),
                                Data::Target(*target_id),
                                Data::Standing(standing_poi_id1, agent_id),
                                Data::FromHandPOI(hand_id_1, agent_id),
                                Data::ToHandPOI(hand_id_2, agent_id),
                                Data::Action(agent_id),
                                Data::PrimitiveAssignment(agent_id, primitive_id1),
                            ];
                            meta_data2 = vec![
                                Data::Simulation,
                                Data::Agent(agent_id),
                                Data::Target(*target_id),
                                Data::Standing(standing_poi_id1, agent_id),
                                Data::FromHandPOI(hand_id_2, agent_id),
                                Data::ToHandPOI(hand_id_1, agent_id),
                                Data::Action(agent_id),
                                Data::PrimitiveAssignment(agent_id, primitive_id2),
                            ];
                        } else {
                            let primitive1 = Primitive::Carry {
                                id: Uuid::new_v4(),
                                target: *target_id,
                                from_standing: standing_poi_id1,
                                to_standing: standing_poi_id2,
                                from_hand: hand_id_1,
                                to_hand: hand_id_2,
                            };

                            let primitive_id1 = primitive1.id();
                            new_names.insert(primitive1.id(), format!("{:?}", primitive1.tag()));
                            self.primitives.insert(primitive_id1, primitive1);

                            let primitive2 = Primitive::Carry {
                                id: Uuid::new_v4(),
                                target: *target_id,
                                from_standing: standing_poi_id2,
                                to_standing: standing_poi_id1,
                                from_hand: hand_id_2,
                                to_hand: hand_id_1,
                            };

                            let primitive_id2 = primitive2.id();
                            new_names.insert(primitive2.id(), format!("{:?}", primitive2.tag()));
                            self.primitives.insert(primitive_id2, primitive2);

                            meta_data1 = vec![
                                Data::Simulation,
                                Data::Agent(agent_id),
                                Data::Target(*target_id),
                                Data::FromStandingPOI(standing_poi_id1, agent_id),
                                Data::ToStandingPOI(standing_poi_id2, agent_id),
                                Data::FromHandPOI(hand_id_1, agent_id),
                                Data::ToHandPOI(hand_id_2, agent_id),
                                Data::Action(agent_id),
                                Data::PrimitiveAssignment(agent_id, primitive_id1),
                            ];
                            meta_data2 = vec![
                                Data::Simulation,
                                Data::Agent(agent_id),
                                Data::Target(*target_id),
                                Data::FromStandingPOI(standing_poi_id2, agent_id),
                                Data::ToStandingPOI(standing_poi_id1, agent_id),
                                Data::FromHandPOI(hand_id_2, agent_id),
                                Data::ToHandPOI(hand_id_1, agent_id),
                                Data::Action(agent_id),
                                Data::PrimitiveAssignment(agent_id, primitive_id2),
                            ];
                        }
                        input1.insert(*place1_id, Signature::Static(1));
                        output1.insert(*place2_id, Signature::Static(1));
                        let transition1 = Transition::new(
                            if is_move { format!(
                                "Move:{}:{}:{}->{}",
                                agent_name,
                                target.name(),
                                hand_poi1.name(),
                                hand_poi2.name()
                            )} else {format!(
                                "Carry:{}:{}:{}->{}",
                                agent_name,
                                target.name(),
                                hand_poi1.name(),
                                hand_poi2.name()
                            )},
                            input1,
                            output1,
                            meta_data1,
                            0.0,
                            vec![],
                        );
                        new_transitions.push(transition1);
                        let mut input2 = existing_reach_transition.output.clone();
                        let mut output2 = existing_reach_transition.input.clone();
                        input2.insert(*place2_id, Signature::Static(1));
                        output2.insert(*place1_id, Signature::Static(1));
                        let transition2 = Transition::new(
                            if is_move { format!(
                                "Move:{}:{}:{}->{}",
                                agent_name,
                                target.name(),
                                hand_poi1.name(),
                                hand_poi2.name()
                            )} else {format!(
                                "Carry:{}:{}:{}->{}",
                                agent_name,
                                target.name(),
                                hand_poi1.name(),
                                hand_poi2.name()
                            )},
                            input2,
                            output2,
                            meta_data2,
                            0.0,
                            vec![],
                        );
                        new_transitions.push(transition2);
                    }
                });
        }

        let mut new_places: Vec<Place> = vec![];
        let all_transitions = net.transitions.clone();
        for (target_id, target) in self.targets.iter() {
            let spawn_transitions = all_transitions.values().filter(|t| t.has_data(&vec![Query::Data(Data::TargetLocationSelected(*target_id))]));
            
            if spawn_transitions.clone().count() > 0 {
                let pre_place = Place::new(
                    format!("Target: {} (pre placed)", target.name()),
                    crate::petri::token::TokenSet::Finite,
                    vec![Data::Target(*target_id), Data::TargetUnplaced(*target_id)],
                );
                let pre_place_id = pre_place.id;
                net.places.insert(pre_place_id, pre_place);
                net.initial_marking.insert(pre_place_id, 1);

                for spawn_transition in spawn_transitions {
                    let hand_location_id = spawn_transition.meta_data.iter().find(|d| d.tag() == DataTag::Hand).unwrap().id().unwrap();
                        let placement_place = Place::new(
                            format!("Target: {} (placed)", target.name()),
                            crate::petri::token::TokenSet::Finite,
                            vec![Data::Target(*target_id), Data::TargetLocationSelected(*target_id)],);
                        let placement_place_id = placement_place.id;

                        let placement_alloc_transition = Transition::new(
                            format!("Locate/Place Part: {}", target.name()),
                            vec![(pre_place_id, Signature::Static(1))]
                                .into_iter()
                                .collect(),
                            vec![(placement_place_id, Signature::Static(1))].into_iter().collect(),
                            vec![
                                Data::Setup,
                                Data::Target(*target_id),
                                Data::TargetLocationSelected(*target_id),
                                Data::AgentAgnostic,
                            ],
                            0.0,
                            vec![],
                        );

                        let new_spawn_transition = spawn_transition
                            .clone()
                            .add_input(&placement_place_id, 1)
                            .add_output(&placement_place_id, 1);

                        new_places.push(placement_place);
                        if (target.pois().is_empty() || target.pois().contains(&hand_location_id)) {
                            new_transitions.push(placement_alloc_transition);
                        }
                        new_transitions.push(new_spawn_transition);
                }   
            }
        }

        for new_place in new_places {
            net.places.insert(new_place.id, new_place);
        }
        for new_transition in new_transitions {
            net.transitions.insert(new_transition.id, new_transition);
        }
        for (name_id, name) in new_names.iter() {
            net.name_lookup.insert(*name_id, name.clone());
        }

        println!(
            "POI Net: Places {:?}, Transitions {:?}",
            net.places.len(),
            net.transitions.len()
        );

        net
    }
}
