use crate::constants::SPLIT_SIZE;
use crate::description::job::Job;
use crate::description::agent::Agent;
use crate::description::primitive::Primitive;
use crate::petri::data::{Data, DataTag, Query};
use crate::petri::net::PetriNet;
use crate::petri::place::Place;
use crate::petri::token::TokenSet;
use crate::petri::transition::{Signature, Transition};
use crate::util::split_primitives;
use itertools::Itertools;
use std::collections::HashMap;
use enum_tag::EnumTag;
use uuid::Uuid;

impl Job {
    pub fn compute_agent_from_basic(&self) -> PetriNet {
        let basic_net = self.basic_net.as_ref().unwrap();
        let mut net = PetriNet::new(basic_net.name.clone());
        net.name_lookup = basic_net.name_lookup.clone();
        net.places = basic_net.places.clone();
        net.initial_marking = basic_net.initial_marking.clone();
        for (agent_id, agent) in self.agents.iter() {
            // Add the agent name to the lookup
            net.name_lookup.insert(*agent_id, agent.name());

            // Add an "indeterminite" place for each agent, representing a limbo state where it hasn't been added.
            let indeterminite_place = Place::new(
                format!("{} ❓", agent.name()),
                TokenSet::Finite,
                vec![Data::Agent(*agent_id), Data::AgentIndeterminite(*agent_id)],
            );
            let indeterminite_place_id: Uuid = indeterminite_place.id;
            net.places
                .insert(indeterminite_place_id, indeterminite_place);
            net.initial_marking.insert(indeterminite_place_id, 1);

            // Add an "initialization" place for each agent, representing where it starts, given that it was added
            let init_place = Place::new(
                agent.name(),
                TokenSet::Finite,
                vec![Data::Agent(*agent_id), Data::AgentSituated(*agent_id)],
            );
            let init_place_id: Uuid = init_place.id;
            net.places.insert(init_place_id, init_place);
            net.initial_marking.insert(init_place_id, 0);

            // Add a "discard" place for each agent, representing the choice to not add it
            let discard_place = Place::new(
                format!("{} 🗑️", agent.name()),
                TokenSet::Sink,
                vec![Data::Agent(*agent_id), Data::AgentDiscard(*agent_id)],
            );
            let discard_place_id: Uuid = discard_place.id;
            net.places.insert(discard_place_id, discard_place);
            net.initial_marking.insert(discard_place_id, 0);

            // Add an "added" place that simply represents that the agent has been added.
            let added_place = Place::new(
                format!("{} ✅", agent.name()),
                TokenSet::Finite,
                vec![Data::Agent(*agent_id), Data::AgentPresent(*agent_id)],
            );
            let added_place_id = added_place.id;
            net.places.insert(added_place_id, added_place);

            // Add transitions from the indeterminite place to the initialization place
            let transition: Transition = Transition::new(
                format!("Add {}", agent.name()),
                vec![(indeterminite_place_id, Signature::Static(1))]
                    .into_iter()
                    .collect(),
                vec![
                    (init_place_id, Signature::Static(1)),
                    (added_place_id, Signature::Static(1)),
                ]
                .into_iter()
                .collect(),
                vec![Data::Agent(*agent_id), Data::AgentAdd(*agent_id)],
                0.0,
                vec![],
            );
            net.transitions.insert(transition.id, transition);

            let transition: Transition = Transition::new(
                format!("Discard {}", agent.name()),
                vec![(indeterminite_place_id, Signature::Static(1))]
                    .into_iter()
                    .collect(),
                vec![(discard_place_id, Signature::Static(1))]
                    .into_iter()
                    .collect(),
                vec![Data::Agent(*agent_id), Data::AgentDiscard(*agent_id)],
                0.0,
                vec![],
            );
            net.transitions.insert(transition.id, transition);
        }

        for transition in basic_net.transitions.values() {
            if transition
                .meta_data
                .iter()
                .map(|d| d.tag())
                .collect::<Vec<DataTag>>()
                .contains(&DataTag::AgentAgnostic)
            {
                let mut t = transition.clone();
                t.id = Uuid::new_v4();
                net.transitions.insert(t.id, t);
            } else {
                let task = transition
                    .meta_data
                    .iter()
                    .find(|d| d.tag() == DataTag::Task)
                    .map(|d| self.tasks.get(&d.id().unwrap()).unwrap())
                    .unwrap();

                let task_id = task.id;

                let pre_allocation_place = Place::new(
                    format!("{}-pre-alloc", transition.name),
                    TokenSet::Finite,
                    vec![Data::Task(task_id), Data::UnnallocatedTask(task_id)],
                );
                let pre_allocation_place_id: Uuid = pre_allocation_place.id;
                net.places
                    .insert(pre_allocation_place_id, pre_allocation_place);
                net.initial_marking.insert(pre_allocation_place_id, 1);

                // New Generalized Creation of Agent Task Transitions
                let mut primitive_assignments: Vec<HashMap<Uuid, Vec<Uuid>>> = vec![];
                for agent_ids in self
                    .agents
                    .keys()
                    .powerset()
                    .filter(|s| s.len() <= SPLIT_SIZE && s.len() > 0)
                {
                    // println!("Agent Ids {:?}", agent_ids);
                    let agents = agent_ids
                        .iter()
                        .map(|id| self.agents.get(id).unwrap())
                        .collect_vec();

                    // Primitive Assignments are a mapping from agent to a set of primitives

                    if agents.len() <= task.primitives.len() {
                        // Assign at least one primitive to each agent
                        let primitive_set: Vec<&Primitive> = task
                            .primitives
                            .iter()
                            .map(|p| self.primitives.get(p).unwrap())
                            .collect();
                        let splits = split_primitives(&primitive_set, agents.len());
                        // println!(
                        //     "Permutations {:?}",
                        //     (0..splits.len()).permutations(splits.len())
                        // );
                        for permutation_assignment in splits.iter().permutations(splits.len()) {
                            // println!("Permutation Assignment {:?}", permutation_assignment);
                            let mut agent_primitive_assignment: HashMap<Uuid, Vec<Uuid>> =
                                HashMap::new();
                            for (idx, assignment) in permutation_assignment.iter().enumerate() {
                                agent_primitive_assignment
                                    .insert(agents[idx].id(), assignment.clone().to_vec());
                            }
                            if !primitive_assignments.contains(&agent_primitive_assignment) {
                                primitive_assignments.push(agent_primitive_assignment);
                            }
                        }
                    } else {
                        // Assign all primitives to the first agent
                        let agent_primitive_assignment: HashMap<Uuid, Vec<Uuid>> =
                            vec![(agents[0].id(), task.primitives.clone())]
                                .into_iter()
                                .collect();
                        if !primitive_assignments.contains(&agent_primitive_assignment) {
                            primitive_assignments.push(agent_primitive_assignment);
                        }
                    }
                }

                for assignment in primitive_assignments {
                    let all_assigned_agent_ids: Vec<&Uuid> = assignment.keys().collect_vec();
                    let all_assigned_agents: Vec<&Agent> = all_assigned_agent_ids
                        .iter()
                        .map(|id| self.agents.get(id).unwrap())
                        .collect();
                    // println!("All assigned agent ids: {:?}", all_assigned_agent_ids);
                    let agent_present_places: Vec<(Uuid, Signature)> = all_assigned_agent_ids
                        .iter()
                        .map(|id| {
                            (
                                net.query_places(&vec![
                                    Query::Data(Data::Agent(**id)),
                                    Query::Data(Data::AgentPresent(**id)),
                                ])
                                .first()
                                .unwrap()
                                .id,
                                Signature::Static(1),
                            )
                        })
                        .collect();

                    let agent_init_places: Vec<(Uuid, Signature)> = all_assigned_agent_ids
                        .iter()
                        .map(|id| {
                            (
                                net.query_places(&vec![
                                    Query::Data(Data::Agent(**id)),
                                    Query::Data(Data::AgentSituated(**id)),
                                ])
                                .first()
                                .unwrap()
                                .id,
                                Signature::Static(1),
                            )
                        })
                        .collect();

                    let allocation_place = Place::new(
                        format!("{}-alloc", transition.name),
                        TokenSet::Finite,
                        vec![Data::Task(task_id), Data::AllocatedTask(task_id)]
                            .into_iter()
                            .chain(
                                all_assigned_agent_ids
                                    .iter()
                                    .map(|id| Data::AgentTaskLock(**id)),
                            )
                            .collect(),
                    );
                    let allocation_place_id = allocation_place.id;
                    net.places.insert(allocation_place_id, allocation_place);

                    let allocation_transition = Transition::new(
                        format!(
                            "{} decide {}",
                            transition.name,
                            all_assigned_agents.iter().map(|a| a.name()).join("+")
                        ),
                        vec![(pre_allocation_place_id, Signature::Static(1))]
                            .into_iter()
                            .chain(agent_present_places.clone().into_iter())
                            .collect(),
                        vec![(allocation_place_id, Signature::Static(1))]
                            .into_iter()
                            .chain(agent_present_places.clone().into_iter())
                            .collect(),
                        vec![Data::Task(task_id), Data::AllocatedTask(task_id)]
                            .into_iter()
                            .chain(all_assigned_agent_ids.iter().map(|id| Data::Agent(**id)))
                            .collect(),
                        0.0,
                        vec![],
                    );
                    net.transitions
                        .insert(allocation_transition.id, allocation_transition);

                    // Add an assignment-specific variant of the transition
                    let mut t = transition.clone();
                    t.id = Uuid::new_v4();
                    t.name = format!(
                        "{}-{}",
                        all_assigned_agents.iter().map(|a| a.name()).join("+"),
                        t.name
                    );
                    for init_place in agent_init_places {
                        t.input.insert(init_place.0, init_place.1.clone());
                        t.output.insert(init_place.0, init_place.1.clone());
                    }
                    t.input.insert(allocation_place_id, Signature::Static(1));
                    t.output.insert(allocation_place_id, Signature::Static(1));

                    for agent_id in all_assigned_agent_ids {
                        t.meta_data.push(Data::Agent(*agent_id));
                        t.meta_data.push(Data::Action(*agent_id));
                        for assigned_primitive_id in assignment.get(agent_id).unwrap() {
                            t.meta_data
                                .push(Data::PrimitiveAssignment(*agent_id, *assigned_primitive_id));
                        }
                    }
                    net.transitions.insert(t.id, t);
                }
            }
        }

        net
    }
}