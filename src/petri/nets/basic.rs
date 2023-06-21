use std::collections::HashMap;
// use std::error::Error;

// use crate::description::target;
use crate::description::task::Task;
use crate::petri::data::Data;
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
                            format!("Int. ({})", process.name), 
                            TokenSet::Finite, 
                            vec![
                                Data::TaskPlace(task.id()),
                                Data::TargetPlace(target.0.id)
                            ]
                        );
                        net.places.insert(place.id, place);
                    }
                }
                Task::Spawn(spawn) => {
                    for target in spawn.output.iter() {
                        let place = Place::new(
                            format!("Spawn {} ({})", target.0.name, spawn.name), 
                            TokenSet::Infinite, 
                            vec![
                                Data::TaskPlace(task.id()),
                                Data::TargetPlace(target.0.id)
                            ]
                        );
                        net.places.insert(place.id, place);
                    }
                }
                Task::Complete(complete) => {
                    for dependency in complete.dependencies.iter() {
                        let place = Place::new(
                            format!("Complete {} ({})", dependency.target.name, complete.name), 
                            TokenSet::Sink, 
                            vec![
                                Data::TaskPlace(task.id()),
                                Data::TargetPlace(dependency.target.id)
                            ]
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
                        let query = vec![
                            Data::TaskPlace(dependency.task.id()),
                            Data::TargetPlace(dependency.target.id)
                        ];
                        let candidates = net.query_places(
                            &query,false);
                        let matching_dep_place = candidates.iter().find(|dep_place| 
                            dep_place.tokens == TokenSet::Infinite || 
                            (dep_place.tokens == TokenSet::Finite && 
                                dep_task.output_target_count(&dependency.target.id) >= dependency.count
                            )
                        );
                        match matching_dep_place {
                            Some(place) => {
                                input.insert(place.id, dependency.count);
                            },
                            None => {
                                return Err(format!("Error Building Basic Net: Dependency for task {} cannot be satisfied. Task {} cannot be found", 
                                    process.name, 
                                    dependency.task.name()
                                ));
                            }
                        }
                    }
                    for (target,count) in process.output.iter() {
                        let matching_places = net.query_places(&vec![Data::TaskPlace(process.id),Data::TargetPlace(target.id)],false);
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
                        name: format!("{}", process.name),
                        input,
                        output,
                        meta_data: vec![
                            Data::TaskTransition(process.id)
                        ]
                    };
        
                    net.transitions.insert(transition.id, transition);
                }
                Task::Complete(complete) => {
                    for dependency in complete.dependencies.iter() {
                        let dep_task_option = tasks.iter().find(|t| t.id()==dependency.task.id());
                        let dep_task: &&Task;
                        match dep_task_option {
                            Some(t) => {
                                dep_task = t;
                            },
                            None => {
                                return Err(format!("Error Building Basic Net: Dependency for task {} cannot be satisfied. Task {} with target {} cannot be found", 
                                        complete.name, 
                                        dependency.task.name(),
                                        dependency.target.name
                                    ));
                            }
                        }
                        let query = vec![
                            Data::TaskPlace(dependency.task.id()),
                            Data::TargetPlace(dependency.target.id)
                        ];
                        let candidates = net.query_places(
                            &query,false);
                        let matching_dep_place = candidates.iter().find(|dep_place| dep_place.tokens == TokenSet::Infinite || (dep_place.tokens == TokenSet::Finite && dep_task.output_target_count(&dependency.target.id) >= dependency.count));
                        match matching_dep_place {
                            Some(place) => {
                                input.insert(place.id, dependency.count);
                            },
                            None => {
                                return Err(format!("Error Building Basic Net: Dependency for task {} cannot be satisfied. Task {} cannot be found", 
                                    complete.name, 
                                    dependency.task.name()
                                ));
                            }
                        }
                        let out_query = vec![Data::TaskPlace(complete.id),Data::TargetPlace(dependency.target.id)];
                        let candidates = net.query_places(&out_query,false);
                        let output_place = candidates.first();
                        match output_place {
                            Some(place) => {
                                output.insert(place.id.clone(), dependency.count);
                            },
                            None => {
                                return Err(format!("Error Building Basic Net: Output for task {} cannot be found.", 
                                        complete.name
                                    ));
                            }
                        };

                    }
                    let transition: Transition = Transition {
                        id: Uuid::new_v4(),
                        name: format!("{}", complete.name),
                        input,
                        output,
                        meta_data: vec![
                            Data::TaskTransition(complete.id),
                            Data::NonAgentTranstion
                        ]
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
