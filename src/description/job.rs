use crate::description::agent::Agent;
use crate::description::poi::PointOfInterest;
use crate::description::primitive::Primitive;
use crate::description::target::Target;
use crate::description::task::Task;
use crate::petri::data::{Data, DataTag, Query};
use crate::petri::net::PetriNet;
use crate::petri::place::Place;
use crate::petri::token::TokenSet;
use crate::petri::transition::Transition;
use enum_tag::EnumTag;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: Uuid,
    pub name: String,
    pub tasks: HashMap<Uuid, Task>,
    pub primitives: HashMap<Uuid, Primitive>,
    pub points_of_interest: HashMap<Uuid, PointOfInterest>,
    pub agents: HashMap<Uuid, Agent>,
    pub targets: HashMap<Uuid, Target>,
    pub basic_net: Option<PetriNet>,
    pub agent_net: Option<PetriNet>,
    pub poi_net: Option<PetriNet>,
}

impl Job {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            tasks: HashMap::new(),
            primitives: HashMap::new(),
            points_of_interest: HashMap::new(),
            agents: HashMap::new(),
            targets: HashMap::new(),
            basic_net: None,
            agent_net: None,
            poi_net: None,
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.insert(task.id, task);
    }

    pub fn add_primitive(&mut self, primitive: Primitive) {
        self.primitives.insert(primitive.id(), primitive);
    }

    pub fn add_point_of_interest(&mut self, poi: PointOfInterest) {
        self.points_of_interest.insert(poi.id(), poi);
    }

    pub fn add_agent(&mut self, agent: Agent) {
        self.agents.insert(agent.id(), agent);
    }

    pub fn add_target(&mut self, target: Target) {
        self.targets.insert(target.id(), target);
    }

    pub fn create_task(&mut self, name: String) -> Uuid {
        let task = Task::new(name);
        let uuid = task.id;
        self.add_task(task);
        uuid
    }

    pub fn create_standing_point_of_interest(
        &mut self,
        name: String,
        x: f64,
        y: f64,
        z: f64,
    ) -> Uuid {
        let poi = PointOfInterest::new_standing(name, x, y, z);
        let uuid = poi.id();
        self.add_point_of_interest(poi);
        uuid
    }

    pub fn create_hand_point_of_interest(&mut self, name: String, x: f64, y: f64, z: f64) -> Uuid {
        let poi = PointOfInterest::new_hand(name, x, y, z);
        let uuid = poi.id();
        self.add_point_of_interest(poi);
        uuid
    }

    pub fn create_robot_agent(
        &mut self,
        name: String,
        reach: f64,        // meters
        payload: f64,      // kg
        agility: f64,      // rating 0-1
        speed: f64,        // m/s
        precision: f64,    // m (repeatability)
        sensing: f64,      // rating 0-1
        mobile_speed: f64, // m/s (zero if not mobile)
    ) -> Uuid {
        let agent = Agent::new_robot(
            name,
            reach,
            payload,
            agility,
            speed,
            precision,
            sensing,
            mobile_speed,
        );
        let uuid = agent.id();
        self.add_agent(agent);
        uuid
    }

    pub fn create_human_agent(&mut self, name: String) -> Uuid {
        let agent = Agent::new_human(name);
        let uuid = agent.id();
        self.add_agent(agent);
        uuid
    }

    pub fn create_precursor_target(&mut self, name: String, size: f64, weight: f64) -> Uuid {
        let target = Target::new_precursor(name, size, weight);
        let uuid = target.id();
        self.add_target(target);
        uuid
    }

    pub fn create_intermediate_target(&mut self, name: String, size: f64, weight: f64) -> Uuid {
        let target = Target::new_intermediate(name, size, weight);
        let uuid = target.id();
        self.add_target(target);
        uuid
    }

    pub fn create_product_target(&mut self, name: String, size: f64, weight: f64) -> Uuid {
        let target = Target::new_product(name, size, weight);
        let uuid = target.id();
        self.add_target(target);
        uuid
    }

    pub fn create_reusable_target(&mut self, name: String, size: f64, weight: f64) -> Uuid {
        let target = Target::new_reusable(name, size, weight);
        let uuid = target.id();
        self.add_target(target);
        uuid
    }

    pub fn add_task_dependency(&mut self, task: Uuid, target: Uuid, count: usize) {
        match self.tasks.get_mut(&task) {
            Some(task_obj) => task_obj.add_dependency(&target, count),
            None => {}
        }
    }

    pub fn add_task_output(&mut self, task: Uuid, target: Uuid, count: usize) {
        match self.tasks.get_mut(&task) {
            Some(task_obj) => task_obj.add_output(&target, count),
            None => {}
        }
    }

    pub fn add_task_point_of_interest(&mut self, task: Uuid, poi: Uuid) {
        match self.tasks.get_mut(&task) {
            Some(task_obj) => task_obj.add_point_of_interest(&poi),
            None => {}
        }
    }

    pub fn add_task_reusable(&mut self, task: Uuid, target: Uuid, count: usize) {
        match self.tasks.get_mut(&task) {
            Some(task_obj) => task_obj.add_reusable(&target, count),
            None => {}
        }
    }

    pub fn add_task_primitive(&mut self, task: Uuid, primitive: Primitive) {
        match self.tasks.get_mut(&task) {
            Some(task_obj) => {
                task_obj.add_primitive(primitive.id());
                self.primitives.insert(primitive.id(), primitive);
            }
            None => {}
        }
    }

    pub fn create_petri_nets(&mut self) {
        self.basic_net = Some(self.create_basic_net());
        self.agent_net = Some(self.create_agent_net());
        self.poi_net = Some(self.create_poi_net());
    }

    pub fn create_basic_net(&mut self) -> PetriNet {
        let mut net: PetriNet = PetriNet::new(self.name.clone());

        for (target_id, target) in self.targets.iter() {
            match target {
                Target::Product { name, .. } => {
                    let place = Place::new(
                        format!("Target: {}", name),
                        TokenSet::Sink,
                        vec![Data::Target(*target_id)],
                    );
                    let place_id = place.id;
                    net.places.insert(place_id, place);
                    net.initial_marking.insert(place_id, 0);
                }
                Target::Intermediate { name, .. } => {
                    let place = Place::new(
                        format!("Target: {}", name),
                        TokenSet::Finite,
                        vec![Data::Target(*target_id)],
                    );
                    let place_id = place.id;
                    net.places.insert(place_id, place);
                    net.initial_marking.insert(place_id, 0);
                }
                Target::Precursor { name, .. } => {
                    let place = Place::new(
                        format!("Target: {}", name),
                        TokenSet::Infinite,
                        vec![Data::Target(*target_id)],
                    );
                    let place_id = place.id;
                    net.places.insert(place_id, place);
                    net.initial_marking.insert(place_id, 0);
                }
                Target::Reusable { name, .. } => {
                    let place = Place::new(
                        format!("Target: {}", name),
                        TokenSet::Finite,
                        vec![Data::Target(*target_id)],
                    );
                    let place_id = place.id;
                    net.places.insert(place_id, place);
                    net.initial_marking.insert(place_id, 1);
                }
            }
            net.name_lookup.insert(*target_id, target.name());
        }

        // Add all dependencies as transitions to the net
        for (task_id, task) in self.tasks.iter() {
            net.name_lookup.insert(*task_id, task.name.clone());
            let mut input: HashMap<Uuid, usize> = HashMap::new();
            let mut output: HashMap<Uuid, usize> = HashMap::new();
            for (dependency_id, count) in &task.dependencies {
                let target_places =
                    net.query_places(&vec![Query::Data(Data::Target(*dependency_id))]);
                for target_place in target_places {
                    input.insert(target_place.id, *count);
                }
            }
            for (output_id, count) in &task.output {
                let target_places = net.query_places(&vec![Query::Data(Data::Target(*output_id))]);
                for target_place in target_places {
                    output.insert(target_place.id, *count);
                }
            }

            let transition: Transition = Transition {
                id: Uuid::new_v4(),
                name: format!("{}", task.name),
                input,
                output,
                meta_data: vec![Data::Task(*task_id)],
            };

            net.transitions.insert(transition.id, transition);
        }

        net
    }

    pub fn create_agent_net(&mut self) -> PetriNet {
        if !self.basic_net.is_some() {
            self.basic_net = Some(self.create_basic_net());
        }
        self.compute_agent_from_basic()
    }

    fn compute_agent_from_basic(&self) -> PetriNet {
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

            // Add transitions from the indeterminite place to the initialization place
            let mut input: HashMap<Uuid, usize> = HashMap::new();
            input.insert(indeterminite_place_id, 1);
            let mut output: HashMap<Uuid, usize> = HashMap::new();
            output.insert(init_place_id, 1);
            let transition: Transition = Transition {
                id: Uuid::new_v4(),
                name: format!("Add {}", agent.name()),
                input,
                output,
                meta_data: vec![Data::Agent(*agent_id), Data::AgentAdd(*agent_id)],
            };
            net.transitions.insert(transition.id, transition);

            let mut input: HashMap<Uuid, usize> = HashMap::new();
            input.insert(indeterminite_place_id, 1);
            let mut output: HashMap<Uuid, usize> = HashMap::new();
            output.insert(discard_place_id, 1);
            let transition: Transition = Transition {
                id: Uuid::new_v4(),
                name: format!("Discard {}", agent.name()),
                input,
                output,
                meta_data: vec![Data::Agent(*agent_id), Data::AgentDiscard(*agent_id)],
            };
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
                let task_id = transition
                    .meta_data
                    .iter()
                    .find(|d| d.tag() == DataTag::Task)
                    .unwrap()
                    .id()
                    .unwrap();
                let pre_allocation_place = Place::new(
                    format!("{}-pre-alloc", transition.name),
                    TokenSet::Finite,
                    vec![Data::Task(task_id), Data::UnnallocatedTask(task_id)],
                );
                let pre_allocation_place_id: Uuid = pre_allocation_place.id;
                net.places
                    .insert(pre_allocation_place_id, pre_allocation_place);
                net.initial_marking.insert(pre_allocation_place_id, 1);
                for (agent_id, agent) in self.agents.iter() {
                    let init_place_id = net
                        .query_places(&vec![
                            Query::Data(Data::Agent(*agent_id)),
                            Query::Data(Data::AgentSituated(*agent_id)),
                        ])
                        .first()
                        .unwrap()
                        .id;

                    let allocation_place = Place::new(
                        format!("{}-alloc", transition.name),
                        TokenSet::Finite,
                        vec![
                            Data::Task(task_id),
                            Data::AllocatedTask(task_id),
                            Data::AgentTaskLock(*agent_id),
                        ],
                    );
                    let allocation_place_id = allocation_place.id;
                    net.places.insert(allocation_place_id, allocation_place);
                    net.initial_marking.insert(allocation_place_id, 0);

                    // Add a transition from the pre-allocation place to the agent's allocation place
                    let allocation_transition: Transition = Transition {
                        id: Uuid::new_v4(),
                        name: format!("{} decide {}", transition.name, agent.name()),
                        input: vec![(pre_allocation_place_id, 1)].into_iter().collect(),
                        output: vec![(allocation_place_id, 1)].into_iter().collect(),
                        meta_data: vec![
                            Data::Task(task_id),
                            Data::Agent(*agent_id),
                            Data::AllocatedTask(task_id),
                        ],
                    };
                    net.transitions
                        .insert(allocation_transition.id, allocation_transition);

                    // Add an agent-specific variant of the transition
                    let mut t = transition.clone();
                    t.id = Uuid::new_v4();
                    t.name = format!("{}-{}", agent.name(), t.name);
                    t.input.insert(init_place_id, 1);
                    t.input.insert(allocation_place_id, 1);
                    t.output.insert(init_place_id, 1);
                    t.output.insert(allocation_place_id, 1);
                    t.meta_data.push(Data::Agent(agent.id()));
                    net.transitions.insert(t.id, t);
                }
            }
        }

        net
    }

    pub fn create_poi_net(&mut self) -> PetriNet {
        if !self.agent_net.is_some() {
            self.agent_net = Some(self.create_agent_net());
        }
        self.compute_poi_from_agent()
    }

    pub fn compute_poi_from_agent(&self) -> PetriNet {
        let agent_net = self.agent_net.as_ref().unwrap();
        let mut net = agent_net.clone();
        let mut standing_pois: Vec<&PointOfInterest> = vec![];
        let mut hand_pois: Vec<&PointOfInterest> = vec![];
        for (poi_id, poi) in self.points_of_interest.iter() {
            match poi {
                PointOfInterest::Standing(_) => standing_pois.push(poi),
                PointOfInterest::Hand(_) => hand_pois.push(poi),
            }
            net.name_lookup.insert(poi_id.clone(), poi.name().clone());
        }
        // println!("Standing POIs: {:#?}", standing_pois);
        // println!("Hand POIs: {:#?}", hand_pois);

        // For each agent spawn location, create a standing place
        for (agent_id, agent) in self.agents.iter() {

            // Determine the pairs of valid standing/hand poi pairs
            let valid_pairs: Vec<(&PointOfInterest,&PointOfInterest)> = standing_pois
                .iter()
                .cartesian_product(&hand_pois)
                .filter(
                    |(standing_poi,hand_poi)| 
                        standing_poi.reachability(hand_poi,agent)
                )
                .map(|(standing_poi,hand_poi)| (*standing_poi,*hand_poi))
                .collect();

            let mut new_transitions: Vec<Transition> = vec![];
            let agent_situated_places =
                agent_net.query_places(&vec![Query::Data(Data::AgentSituated(*agent_id))]);
            for agent_situated_place in agent_situated_places {
                net.split_place(
                    &agent_situated_place.id,
                    valid_pairs
                        .iter()
                        .map(|(standing_poi, hand_poi)| 
                            vec![
                                Data::POI(standing_poi.id()),
                                Data::POI(hand_poi.id()),
                                Data::Standing(standing_poi.id()),
                                Data::Hand(hand_poi.id()),
                            ])
                        .collect(),
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
                            let transition1 = Transition {
                                id: Uuid::new_v4(),
                                name: format!("{}:Reach:{}->{}", agent.name(), hand_poi1.name(), hand_poi2.name()),
                                input: vec![(place1.id, 1)].into_iter().collect(),
                                output: vec![(place2.id, 1)].into_iter().collect(),
                                meta_data: vec![
                                    Data::Agent(*agent_id),
                                    Data::POI(standing_poi_id1),
                                    Data::POI(hand_poi_id1),
                                    Data::POI(hand_poi_id2),
                                    Data::Standing(standing_poi_id1),
                                    Data::FromHandPOI(hand_poi_id1),
                                    Data::ToHandPOI(hand_poi_id2),
                                ],
                            };
                            new_transitions.push(transition1);
                            let transition2 = Transition {
                                id: Uuid::new_v4(),
                                name: format!("{}:Reach:{}->{}", agent.name(), hand_poi2.name(), hand_poi1.name()),
                                input: vec![(place2.id, 1)].into_iter().collect(),
                                output: vec![(place1.id, 1)].into_iter().collect(),
                                meta_data: vec![
                                    Data::Agent(*agent_id),
                                    Data::POI(standing_poi_id1),
                                    Data::POI(hand_poi_id1),
                                    Data::POI(hand_poi_id2),
                                    Data::Standing(standing_poi_id1),
                                    Data::FromHandPOI(hand_poi_id2),
                                    Data::ToHandPOI(hand_poi_id1),
                                ],
                            };
                            new_transitions.push(transition2);
                        }
                    } else if hand_poi1 == hand_poi2 {
                        // This is strictly a travel
                        if standing_poi1.reachability(hand_poi2, agent) {
                            let transition1 = Transition {
                                id: Uuid::new_v4(),
                                name: format!("{}:Travel:{}->{}", agent.name(), standing_poi1.name(), standing_poi2.name()),
                                input: vec![(place1.id, 1)].into_iter().collect(),
                                output: vec![(place2.id, 1)].into_iter().collect(),
                                meta_data: vec![
                                    Data::Agent(*agent_id),
                                    Data::POI(standing_poi_id1),
                                    Data::POI(standing_poi_id2),
                                    Data::POI(hand_poi_id1),
                                    Data::Hand(hand_poi_id1),
                                    Data::FromStandingPOI(standing_poi_id1),
                                    Data::ToStandingPOI(standing_poi_id2),
                                ],
                            };
                            new_transitions.push(transition1);
                            let transition2 = Transition {
                                id: Uuid::new_v4(),
                                name: format!("{}:Travel:{}->{}", agent.name(), standing_poi2.name(), standing_poi1.name()),
                                input: vec![(place2.id, 1)].into_iter().collect(),
                                output: vec![(place1.id, 1)].into_iter().collect(),
                                meta_data: vec![
                                    Data::Agent(*agent_id),
                                    Data::POI(standing_poi_id1),
                                    Data::POI(standing_poi_id2),
                                    Data::POI(hand_poi_id1),
                                    Data::Hand(hand_poi_id1),
                                    Data::FromStandingPOI(standing_poi_id2),
                                    Data::ToStandingPOI(standing_poi_id1),
                                ],
                            };
                            new_transitions.push(transition2);
                        }
                    }
                    
                });
            }


            for transition in new_transitions {
                net.transitions.insert(transition.id, transition);
            }
        }

        // Refine the task transitions to include only those POIs defined in the task (if applicable)
        net.transitions.retain(|_, transition| {
            if transition.meta_data.iter().any(|d| d.tag() == DataTag::Task) && transition.meta_data.iter().any(|d| d.tag() == DataTag::Hand) {
                let task_id = transition.meta_data.iter().find(|d| d.tag() == DataTag::Task).unwrap().id().unwrap();
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
            }
            true
        });

        net
    }
}
