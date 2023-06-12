use std::collections::HashMap;
// use std::error::Error;

// use crate::description::target;
use crate::description::task::Task;
// use crate::description::target::Target;
use crate::petri::place::Place;
use crate::petri::transition::Transition;
use crate::petri::token::TokenSet;
use crate::petri::nets::net::PetriNet;
use uuid::Uuid;
// use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct BasicNet<'a> {
    pub id: Uuid,
    pub name: String,
    pub places: HashMap<Uuid, Place>,
    pub transitions: HashMap<Uuid, Transition>,
    pub tasks: HashMap<Uuid, Task<'a>>,
}

impl<'a> BasicNet<'a> {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            places: HashMap::new(),
            transitions: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    pub fn from_tasks(name: String, tasks: Vec<&'a Task<'a>>) -> Result<Self, String> {
        let mut net: BasicNet = Self::new(name);

        // Add all tasks as places to the net
        for task in tasks.iter() {
            match task {
                Task::Process(process) => {
                    for target in process.output.iter() {
                        let place = Place::new(
                            format!("place-{}-{}", target.0.name, process.name), 
                            TokenSet::Finite, 
                            Some(task.id()),
                            Some(target.0.id)
                        );
                        net.places.insert(place.id, place);
                    }
                }
                Task::Spawn(spawn) => {
                    for target in spawn.output.iter() {
                        let place = Place::new(
                            format!("place-{}-{}", target.0.name, spawn.name), 
                            TokenSet::Infinite, 
                            Some(task.id()),
                            Some(target.0.id),
                        );
                        net.places.insert(place.id, place);
                    }
                }
                Task::Complete(complete) => {
                    for dependency in complete.dependencies.iter() {
                        let place = Place::new(
                            format!("place-{}", complete.name), 
                            TokenSet::Sink, 
                            Some(task.id()),
                            Some(dependency.target.id),
                        );
                        net.places.insert(place.id, place);
                    }
                }
            }
        }

        // Add all dependencies as transitions to the net
        for task in tasks.iter() {
            let mut input: HashMap<Uuid, usize> = HashMap::new();
            let mut output: HashMap<Uuid, usize> = HashMap::new();
            match task {
                Task::Spawn(_) => {
                    continue;
                },
                Task::Process(process) => {
                    for dependency in process.dependencies.iter() {
                        let dep_task_option = tasks.iter().find(|t| t.id()==dependency.task.id());
                        let dep_task: &&Task;
                        match dep_task_option {
                            Some(t) => {
                                dep_task = t;
                            },
                            None => {
                                return Err(format!("Error Building Basic Net: Dependency for task {} cannot be satisfied. Task {} with target {} cannot be found", 
                                        process.name, 
                                        dependency.task.name(),
                                        dependency.target.name
                                    ));
                            }
                        }
                        let matching_dep_places = net.query_places(Some(dependency.task.id()), Some(dependency.target.id));
                        if !matching_dep_places.is_empty() {
                            let place = matching_dep_places[0];
                            if place.tokens == TokenSet::Infinite {
                                input.insert(place.id, dependency.count);
                            } else if place.tokens == TokenSet::Finite && place.source_task.map_or_else(|| false, |_t| {
                                dep_task.output_target_count(&dependency.target.id) >= dependency.count
                            }) {
                                input.insert(place.id, dependency.count);
                            } else {
                                return Err(format!("Error Building Basic Net: Dependency for task {} cannot be satisfied. Task {}, which is a dependency, only produces {} of {} ({} needed)", 
                                    process.name, 
                                    dependency.task.name(), 
                                    dependency.task.output_target_count(&dependency.target.id), 
                                    dependency.target.name, 
                                    dependency.count
                                ));
                            }
                        } else {
                            return Err(format!("Error Building Basic Net: Dependency for task {} cannot be satisfied. Task {} cannot be found", 
                                    process.name, 
                                    dependency.task.name()
                                ));
                        }
                    }
                    for (target,count) in process.output.iter() {
                        let matching_places = net.query_places(Some(process.id), Some(target.id));
                        match matching_places.first() {
                            Some(place) => {
                                output.insert(place.id, *count);
                            },
                            None => {
                                return Err(format!("Error Building Basic Net: Output for task {} cannot be satisfied. Task {} cannot be found", 
                                        process.name, 
                                        target.name
                                    ));
                            }
                        };
                    }
                    let transition: Transition = Transition {
                        id: Uuid::new_v4(),
                        name: format!("transition-{}", process.name),
                        input,
                        output,
                        source_task: Some(process.id),
                    };
        
                    net.transitions.insert(transition.id, transition);
                }
                Task::Complete(complete) => {
                    for dependency in complete.dependencies.iter() {
                        let dep_task = tasks.iter().find(|t| t.id()==dependency.task.id()).unwrap();
                        let matching_dep_places = net.query_places(Some(dependency.task.id()), Some(dependency.target.id));
                        if !matching_dep_places.is_empty() {
                            let place = matching_dep_places[0];
                            if place.tokens == TokenSet::Infinite {
                                input.insert(place.id, dependency.count);
                            } else if place.tokens == TokenSet::Finite && place.source_task.map_or_else(|| false, |_t| {
                                dep_task.output_target_count(&dependency.target.id) >= dependency.count
                            }) {
                                input.insert(place.id, dependency.count);
                            } else {
                                return Err(format!("Error Building Basic Net: Dependency for task {} cannot be satisfied. Task {}, which is a dependency, only produces {} of {} ({} needed)", 
                                    complete.name, 
                                    dependency.task.name(), 
                                    dependency.task.output_target_count(&dependency.target.id), 
                                    dependency.target.name, 
                                    dependency.count
                                ));
                            }
                        } else {
                            return Err(format!("Error Building Basic Net: Dependency for task {} cannot be satisfied. Task {} with target {} cannot be found", 
                                    complete.name, 
                                    dependency.task.name(),
                                    dependency.target.name
                                ));
                        }
                        // Create the post-transition node connections based on the dependencies
                        let associated_places = net.query_places(Some(complete.id), Some(dependency.target.id));
                        if !associated_places.is_empty() {
                            output.insert(associated_places[0].id, dependency.count);
                        } else {
                            return Err(format!("Error Building Basic Net: Output place for completion task {} cannot be found!", 
                                    complete.name
                                ));
                        }

                    }
                    let transition: Transition = Transition {
                        id: Uuid::new_v4(),
                        name: format!("transition-{}", complete.name,),
                        input,
                        output,
                        source_task: Some(complete.id),
                    };
        
                    net.transitions.insert(transition.id, transition);
                }
            }
            
        }

        for task in tasks {
            net.tasks.insert(task.id(), task.clone());
        }

        Ok(net)
    }

    pub fn query_places(&self, task: Option<Uuid>, target: Option<Uuid>) -> Vec<&Place> {
        self.places.values().into_iter()
            .filter(|place| task.is_none() || place.source_task == task)
            .filter(|place| target.is_none() || place.target_id == target)
            .collect()
    }

    pub fn transitions_derived_from_task(&self, task: Uuid) -> Vec<&Transition> {
        self.transitions.values().into_iter().filter(|transition| transition.source_task == Some(task)).collect()
    }

    pub fn places_derived_from_task(&self, task: Uuid) -> Vec<&Place> {
        self.places.values().into_iter().filter(|place| place.source_task == Some(task)).collect()
    }

    pub fn transitions_between(&mut self, source: Uuid, target: Uuid, source_task: Option<Uuid>) -> Vec<&mut Transition> {
        let mut transitions: Vec<&mut Transition> = vec![];
        for transition in self.transitions.values_mut() {
            if transition.input.contains_key(&source) && transition.output.contains_key(&target) {
                match source_task {
                    Some(task) => {
                        if task == transition.source_task.unwrap() {
                            transitions.push(transition)
                        }
                    }
                    None => transitions.push(transition)
                }
            }
        }
        transitions
    }
}

impl <'a> PetriNet<'a> for BasicNet<'a> {

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
        self.transitions.values().for_each(|transition: &Transition| {
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
        self.transitions.values_mut().for_each(|transition: &mut Transition| {
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

// impl <'a> fmt::Display for BasicNet<'a> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "BasicNet {} ({}): {{\n", self.name, self.id)?;
//         if self.places.is_empty() {
//             write!(f, "\tPlaces: [],\n")?;
//         } else {
//             write!(f, "\tPlaces: [\n")?;
//             for place in self.places.values() {
//                 write!(f, "\t\t{}: {{ name: {}, tokens: {:?}, source_task: {:?}}},\n", place.id, place.name, place.tokens, place.source_task)?;
//             }
//             write!(f, "\t],\n")?;
//         }

//         if self.transitions.is_empty() {
//             write!(f, "\tTransitions: [],\n")?;
//         } else {
//             write!(f, "\tTransitions: [\n")?;
//             for transition in self.transitions.values() {
//                 write!(f, "\t\t{}: {{ name: {}, input: {:?}, output: {:?}, source_task: {:?}}},\n", transition.id, transition.name, transition.input,  transition.output, transition.source_task)?;
//             }
//             write!(f, "\t],\n")?;
//         }
       
//         write!(f, "}}\n")
//     }
// }
