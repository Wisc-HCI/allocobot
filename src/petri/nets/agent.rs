use std::collections::HashMap;
use std::vec;
// use std::error::Error;

// use crate::description::target;
use crate::description::agent::Agent;
use crate::description::task::Task;
use crate::petri::data::Data;
// use crate::description::target::Target;
use crate::petri::nets::basic::BasicNet;
use crate::petri::nets::net::PetriNet;
use crate::petri::place::Place;
use crate::petri::token::TokenSet;
use crate::petri::transition::Transition;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct AgentNet<'a> {
    pub id: Uuid,
    pub name: String,
    pub places: HashMap<Uuid, Place>,
    pub transitions: HashMap<Uuid, Transition>,
    pub tasks: HashMap<Uuid, Task<'a>>,
    pub agents: HashMap<Uuid, Agent>,
}

impl<'a> AgentNet<'a> {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            places: HashMap::new(),
            transitions: HashMap::new(),
            tasks: HashMap::new(),
            agents: HashMap::new(),
        }
    }
}

impl<'a> From<(BasicNet<'a>, Vec<Agent>)> for AgentNet<'a> {
    fn from(value: (BasicNet<'a>, Vec<Agent>)) -> Self {
        let mut net = AgentNet::new(value.0.name.clone());
        net.id = value.0.id;
        net.places = value.0.places;
        net.tasks = value.0.tasks;
        let mut non_agent_transitions: Vec<Uuid> = vec![];
        for agent in value.1.iter() {
            let place = Place::new(
                agent.name(),
                TokenSet::Finite,
                vec![Data::AgentLock(agent.id())],
            );
            let place_id: Uuid = place.id;
            net.places.insert(place.id, place);
            for transition in value.0.transitions.values() {
                if transition.has_data(&vec![Data::NonAgentTranstion], true)
                    && non_agent_transitions.contains(&transition.id)
                {
                    continue;
                } else if transition.has_data(&vec![Data::NonAgentTranstion], true) {
                    non_agent_transitions.push(transition.id);
                    let mut t = transition.clone();
                    t.id = Uuid::new_v4();
                    net.transitions.insert(t.id, t);
                } else {
                    let mut t = transition.clone();
                    t.id = Uuid::new_v4();
                    t.name = format!("{}-{}", agent.name(), t.name);
                    t.input.insert(place_id, 1);
                    t.output.insert(place_id, 1);
                    t.meta_data.push(Data::AgentTransition(agent.id()));
                    net.transitions.insert(t.id, t);
                }
            }
        }
        for transition in value.0.transitions.values() {
            if !transition.has_data(&vec![Data::NonAgentTranstion], true) {
                let transition_lock_place = Place::new(
                    format!("{}-lock", transition.name),
                    TokenSet::Finite,
                    vec![Data::TaskLock(transition.id)],
                );
                let lock_id: Uuid = transition_lock_place.id;
                let transition_nodes = net.transitions_derived_from_task(
                    transition
                        .meta_data
                        .iter()
                        .find(|d| d.is_task_transition())
                        .unwrap()
                        .id()
                        .unwrap(),
                );
    
                for transition_node in transition_nodes {
                    transition_node.input.insert(lock_id, 1);
                    transition_node.output.insert(lock_id, 1);
                }
                net.places.insert(lock_id, transition_lock_place);
            }
            
        }
        net.agents = value.1.into_iter().map(|a| (a.id(), a)).collect();

        // println!("AgentNet Transitions: {:#?}", net.transitions);
        // println!("AgentNet Places: {:#?}", net.places);
        net
    }
}

impl<'a> PetriNet<'a> for AgentNet<'a> {
    fn get_id(&self) -> &Uuid {
        &self.id
    }

    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_places(&self) -> HashMap<Uuid, &Place> {
        let mut places: HashMap<Uuid, &Place> = HashMap::new();
        self.places.values().for_each(|place: &Place| {
            places.insert(place.id, place);
        });
        places
    }

    fn get_transitions(&self) -> HashMap<Uuid, &Transition> {
        let mut transitions: HashMap<Uuid, &Transition> = HashMap::new();
        self.transitions
            .values()
            .for_each(|transition: &Transition| {
                transitions.insert(transition.id, transition);
            });
        transitions
    }

    fn get_tasks(&self) -> HashMap<Uuid, &Task<'a>> {
        let mut tasks: HashMap<Uuid, &Task<'a>> = HashMap::new();
        self.tasks.values().for_each(|task: &Task| {
            tasks.insert(task.id(), task);
        });
        tasks
    }

    fn get_places_mut(&mut self) -> HashMap<Uuid, &mut Place> {
        let mut places: HashMap<Uuid, &mut Place> = HashMap::new();
        self.places.values_mut().for_each(|place: &mut Place| {
            places.insert(place.id, place);
        });
        places
    }

    fn get_transitions_mut(&mut self) -> HashMap<Uuid, &mut Transition> {
        let mut transitions: HashMap<Uuid, &mut Transition> = HashMap::new();
        self.transitions
            .values_mut()
            .for_each(|transition: &mut Transition| {
                transitions.insert(transition.id, transition);
            });
        transitions
    }

    fn get_tasks_mut(&mut self) -> HashMap<Uuid, &mut Task<'a>> {
        let mut tasks: HashMap<Uuid, &mut Task> = HashMap::new();
        self.tasks.values_mut().for_each(|task: &mut Task| {
            tasks.insert(task.id(), task);
        });
        tasks
    }
}

impl<'a> AgentNet<'a> {
    pub fn transitions_derived_from_task(&mut self, task: Uuid) -> Vec<&mut Transition> {
        self.transitions
            .values_mut()
            .into_iter()
            .filter(|transition| transition.meta_data.contains(&Data::TaskTransition(task)))
            .collect()
    }

    pub fn transitions_associated_with_agent(&mut self, agent: Uuid) -> Vec<&mut Transition> {
        self.transitions
            .values_mut()
            .into_iter()
            .filter(|transition| transition.meta_data.contains(&Data::AgentTransition(agent)))
            .collect()
    }

    pub fn transitions_connected_to_place(&mut self, place: Uuid) -> Vec<&mut Transition> {
        self.transitions
            .values_mut()
            .into_iter()
            .filter(|transition| {
                transition.input.contains_key(&place) || transition.output.contains_key(&place)
            })
            .collect()
    }
}
