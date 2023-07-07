use crate::description::agent::Agent;
use crate::description::poi::PointOfInterest;
use crate::description::primitive::Primitive;
use crate::description::target::Target;
use crate::description::task::Task;
use crate::petri::data::{Data, DataTag};
use crate::petri::net::PetriNet;
use crate::petri::place::Place;
use crate::petri::token::TokenSet;
use crate::petri::transition::Transition;
use enum_tag::EnumTag;
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
            name, reach, payload, agility, speed, precision, sensing, mobile_speed,
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
        }

        // Add all dependencies as transitions to the net
        for (task_id, task) in self.tasks.iter() {
            let mut input: HashMap<Uuid, usize> = HashMap::new();
            let mut output: HashMap<Uuid, usize> = HashMap::new();
            for (dependency_id, count) in &task.dependencies {
                let target_places = net.query_places(&vec![Data::Target(*dependency_id)], false);
                for target_place in target_places {
                    input.insert(target_place.id, *count);
                }
            }
            for (output_id, count) in &task.output {
                let target_places = net.query_places(&vec![Data::Target(*output_id)], false);
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
        net.places = basic_net.places.clone();
        net.initial_marking = basic_net.initial_marking.clone();
        for (agent_id, agent) in self.agents.iter() {
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
                        .query_places(
                            &vec![Data::Agent(*agent_id), Data::AgentSituated(*agent_id)],
                            false,
                        )
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

    pub fn compute_poi_from_agent(&self) -> PetriNet {
        let agent_net = self.agent_net.as_ref().unwrap();
        let mut net = PetriNet::new(agent_net.name.clone());

        let standing_pois: Vec<&PointOfInterest> = self.points_of_interest.values().filter(|poi| poi.is_standing()).collect::<Vec<&PointOfInterest>>();
        let hand_pois: Vec<&PointOfInterest> = self.points_of_interest.values().filter(|poi| poi.is_hand()).collect::<Vec<&PointOfInterest>>();

        // For each agent spawn location, create a standing place
        for (agent_id, agent) in self.agents.iter() {
            let agent_situated_places = agent_net.query_places(&vec![Data::AgentSituated(*agent_id)], false);
            for agent_situated_place in agent_situated_places {
                net.split_place(&agent_situated_place.id, standing_pois.iter().map(|poi| vec![Data::POI(poi.id())]).collect());
                
            }
        }
        


        net
    }
}
