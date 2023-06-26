use std::collections::HashMap;
use uuid::Uuid;
use crate::description::task::Task;
use crate::description::primitive::Primitive;
use crate::description::poi::PointOfInterest;
use crate::description::agent::Agent;
use crate::description::target::Target;
use crate::petri::net::PetriNet;
use crate::petri::place::Place;
use crate::petri::transition::Transition;
use crate::petri::token::TokenSet;
use crate::petri::data::{Data,DataTag};
use enum_tag::EnumTag;

pub struct Job {
    pub id: Uuid,
    pub name: String,
    pub tasks: HashMap<Uuid,Task>,
    pub primitives: HashMap<Uuid,Primitive>,
    pub points_of_interest: HashMap<Uuid,PointOfInterest>,
    pub agents: HashMap<Uuid,Agent>,
    pub targets: HashMap<Uuid,Target>,
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
            agent_net: None
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.insert(task.id(),task);
    }

    pub fn add_primitive(&mut self, primitive: Primitive) {
        self.primitives.insert(primitive.id(),primitive);
    }

    pub fn add_point_of_interest(&mut self, poi: PointOfInterest) {
        self.points_of_interest.insert(poi.id(),poi);
    }

    pub fn add_agent(&mut self, agent: Agent) {
        self.agents.insert(agent.id(),agent);
    }

    pub fn add_target(&mut self, target: Target) {
        self.targets.insert(target.id, target);
    }

    pub fn create_spawn_task(&mut self, name: String) -> Uuid {
        let mut task = Task::new_spawn();
        let uuid = task.id();
        task.set_name(&name);
        self.add_task(task);
        uuid
    }

    pub fn create_process_task(&mut self, name: String) -> Uuid {
        let mut task = Task::new_process();
        let uuid = task.id();
        task.set_name(&name);
        self.add_task(task);
        uuid
    }

    pub fn create_complete_task(&mut self, name: String) -> Uuid {
        let mut task = Task::new_complete();
        let uuid = task.id();
        task.set_name(&name);
        self.add_task(task);
        uuid
    }

    // pub fn create_primitive(&mut self, name: String) -> &Uuid {
    //     let primitive = Primitive::new(name);
    //     self.add_primitive(primitive);
    //     &primitive.id
    // }

    pub fn create_standing_point_of_interest(&mut self, name: String, x: f64, y: f64, z: f64) -> Uuid {
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

    pub fn create_robot_agent(&mut self, 
        name: String,
        reach: f64,     // meters
        payload: f64,   // kg
        agility: f64,   // rating 0-1
        speed: f64,     // m/s
        precision: f64, // m (repeatability)
        sensing: f64,   // rating 0-1
        mobile: bool    // true/false);
    ) -> Uuid {
        let agent = Agent::new_robot(
            name,
            reach,
            payload,
            agility,
            speed,
            precision,
            sensing,
            mobile
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

    pub fn create_target(&mut self, name: String, size: f64, weight: f64) -> Uuid {
        let target = Target::new(name, size, weight);
        let uuid = target.id;
        self.add_target(target);
        uuid
    }

    pub fn add_task_dependency(&mut self, task: Uuid, dependent: Uuid, target: Uuid) {
        match self.tasks.get_mut(&task) {
            Some(task_obj) => {
                task_obj.add_dependency(&dependent, &target)
            },
            None => {}
        }
    }

    pub fn add_task_output(&mut self, task: Uuid, target: Uuid, count: usize) {
        match self.tasks.get_mut(&task) {
            Some(task_obj) => {
                task_obj.add_output(&target, count)
            },
            None => {}
        }
    }

    pub fn add_task_primitive(&mut self, task: Uuid, primitive: Primitive) {
        match self.tasks.get_mut(&task) {
            Some(task_obj) => {
                task_obj.add_primitive(primitive.id());
                self.primitives.insert(primitive.id(),primitive);
            },
            None => {

            }
        }
    }

    pub fn create_basic_net(&mut self) -> Result<String,String> {
        let mut net: PetriNet = PetriNet::new(self.name.clone());

        // Add all tasks as places to the net
        for (task_id, task) in self.tasks.iter() {
            match task {
                Task::Process(process) => {
                    for (target, _count) in process.output.iter() {
                        let place = Place::new(
                            format!("Int. {} ({})", self.targets.get(target).map(|t| &t.name).unwrap_or(&"Unknown".into()), process.name), 
                            TokenSet::Finite, 
                            vec![
                                Data::TaskPlace(*task_id),
                                Data::TargetPlace(*target)
                            ]
                        );
                        net.places.insert(place.id, place);
                    }
                }
                Task::Spawn(spawn) => {
                    for (target, _count) in spawn.output.iter() {
                        let place = Place::new(
                            format!("Spawn {} ({})", self.targets.get(target).map(|t| &t.name).unwrap_or(&"Unknown".into()), spawn.name), 
                            TokenSet::Infinite, 
                            vec![
                                Data::TaskPlace(*task_id),
                                Data::TargetPlace(*target)
                            ]
                        );
                        net.places.insert(place.id, place);
                    }
                }
                Task::Complete(complete) => {
                    for dependency in complete.dependencies.iter() {
                        let place = Place::new(
                            format!("Complete {} ({})", self.targets.get(&dependency.target).map(|t| &t.name).unwrap_or(&"Unknown".into()), complete.name), 
                            TokenSet::Sink, 
                            vec![
                                Data::TaskPlace(task.id()),
                                Data::TargetPlace(dependency.target)
                            ]
                        );
                        net.places.insert(place.id, place);
                    }
                }
            }
        }

        // Add all dependencies as transitions to the net
        for (task_id, task) in self.tasks.iter() {
            let mut input: HashMap<Uuid, usize> = HashMap::new();
            let mut output: HashMap<Uuid, usize> = HashMap::new();
            match task {
                Task::Spawn(_) => {
                    continue;
                },
                Task::Process(process) => {
                    for dependency in process.dependencies.iter() {
                        let dep_task_option = self.tasks.get(&dependency.task);
                        let dep_task: &Task;
                        match dep_task_option {
                            Some(t) => {
                                dep_task = t;
                            },
                            None => {
                                return Err(format!("Error Building Basic Net: Dependency for task {} cannot be satisfied. Task {} with target {} cannot be found", 
                                        process.name, 
                                        self.tasks.get(&dependency.task).map(|t| t.name()).unwrap_or("Unknown".into()),
                                        self.targets.get(&dependency.target).map(|t| &t.name).unwrap_or(&"Unknown".into())
                                    ));
                            }
                        }
                        let query = vec![
                            Data::TaskPlace(dependency.task),
                            Data::TargetPlace(dependency.target)
                        ];
                        let candidates = net.query_places(
                            &query,false);
                        let matching_dep_place = candidates.iter().find(|dep_place| 
                            dep_place.tokens == TokenSet::Infinite || 
                            (dep_place.tokens == TokenSet::Finite && 
                                dep_task.output_target_count(&dependency.target) >= dependency.count
                            )
                        );
                        match matching_dep_place {
                            Some(place) => {
                                input.insert(place.id, dependency.count);
                            },
                            None => {
                                return Err(format!("Error Building Basic Net: Dependency for task {} cannot be satisfied. Task {} cannot be found", 
                                    process.name, 
                                    self.tasks.get(&dependency.task).map(|t| t.name()).unwrap_or("Unknown".into())
                                ));
                            }
                        }
                    }
                    for (target,count) in process.output.iter() {
                        let matching_places = net.query_places(&vec![Data::TaskPlace(*task_id),Data::TargetPlace(*target)],false);
                        match matching_places.first() {
                            Some(place) => {
                                output.insert(place.id, *count);
                            },
                            None => {
                                return Err(format!("Error Building Basic Net: Output for task {} cannot be satisfied. Task {} cannot be found", 
                                        process.name, 
                                        self.targets.get(target).map(|t| &t.name).unwrap_or(&"Unknown".into())
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
                            Data::TaskTransition(*task_id)
                        ]
                    };
        
                    net.transitions.insert(transition.id, transition);
                }
                Task::Complete(complete) => {
                    for dependency in complete.dependencies.iter() {
                        let dep_task_option = self.tasks.get(&dependency.task);
                        let dep_task: &Task;
                        match dep_task_option {
                            Some(t) => {
                                dep_task = t;
                            },
                            None => {
                                return Err(format!("Error Building Basic Net: Dependency for task {} cannot be satisfied. Task {} with target {} cannot be found", 
                                        complete.name, 
                                        self.tasks.get(&dependency.task).map(|t| t.name()).unwrap_or("Unknown".into()),
                                        self.targets.get(&dependency.target).map(|t| &t.name).unwrap_or(&"Unknown".into())
                                    ));
                            }
                        }
                        let query = vec![
                            Data::TaskPlace(dependency.task),
                            Data::TargetPlace(dependency.target)
                        ];
                        let candidates = net.query_places(
                            &query,false);
                        let matching_dep_place = candidates.iter().find(|dep_place| dep_place.tokens == TokenSet::Infinite || (dep_place.tokens == TokenSet::Finite && dep_task.output_target_count(&dependency.target) >= dependency.count));
                        match matching_dep_place {
                            Some(place) => {
                                input.insert(place.id, dependency.count);
                            },
                            None => {
                                return Err(format!("Error Building Basic Net: Dependency for task {} cannot be satisfied. Task {} cannot be found", 
                                    complete.name, 
                                    self.tasks.get(&dependency.task).map(|t| t.name()).unwrap_or("Unknown".into()),
                                ));
                            }
                        }
                        let out_query = vec![Data::TaskPlace(*task_id),Data::TargetPlace(dependency.target)];
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
                            Data::TaskTransition(*task_id),
                            Data::NonAgentTransition
                        ]
                    };
        
                    net.transitions.insert(transition.id, transition);
                }
            }
            
        }

        self.basic_net = Some(net);
        Ok("created basic net".into())
    }

    pub fn create_agent_net(&mut self) -> Result<String,String> {
        if self.basic_net.is_some() {
            self.agent_net = Some(self.compute_agent_from_basic());
            Ok("created agent net".into())
        } else {
            let result = self.create_basic_net();
            match result {
                Ok(_) => {
                    self.agent_net = Some(self.compute_agent_from_basic());
                    Ok("created agent net".into())
                },
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    fn compute_agent_from_basic(&self) -> PetriNet {
        let basic_net = self.basic_net.as_ref().unwrap();
        let mut net = PetriNet::new(basic_net.name.clone());
        net.places = basic_net.places.clone();
        for (agent_id, agent) in self.agents.iter() {
            // Add an "indeterminite" place for each agent, representing a limbo state where it hasn't been added. 
            let indeterminite_place = Place::new(
                format!("{} ❓",agent.name()),
                TokenSet::Finite,
                vec![Data::AgentIndeterminitePlace(*agent_id)],
            );
            let indeterminite_place_id: Uuid = indeterminite_place.id;
            net.places.insert(indeterminite_place.id, indeterminite_place);

            // Add an "initialization" place for each agent, representing where it starts, given that it was added
            let init_place = Place::new(
                agent.name(),
                TokenSet::Finite,
                vec![Data::AgentInitialPlace(*agent_id)],
            );
            let init_place_id: Uuid = init_place.id;
            net.places.insert(init_place.id, init_place);

            // Add a "discard" place for each agent, representing the choice to not add it
            let discard_place = Place::new(
                format!("{} 🗑️",agent.name()),
                TokenSet::Sink,
                vec![Data::AgentDiscardPlace(*agent_id)],
            );
            let discard_place_id: Uuid = discard_place.id;
            net.places.insert(discard_place.id, discard_place);

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
                meta_data: vec![
                    Data::AgentTransition(*agent_id),
                    Data::AgentAddTransition(*agent_id),
                ],
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
                meta_data: vec![
                    Data::AgentTransition(*agent_id),
                    Data::AgentDiscardTransition(*agent_id),
                ],
            };
            net.transitions.insert(transition.id, transition);
        }

        for transition in basic_net.transitions.values() {
            if transition.meta_data.iter().map(|d| d.tag()).collect::<Vec<DataTag>>().contains(&DataTag::NonAgentTransition) {
                let mut t = transition.clone();
                t.id = Uuid::new_v4();
                net.transitions.insert(t.id, t);
            } else {
                let task_id = transition.meta_data.iter().find(|d| d.tag() == DataTag::TaskTransition).unwrap().id().unwrap();
                let pre_allocation_place = Place::new(
                    format!("{}-pre-alloc", transition.name),
                    TokenSet::Finite,
                    vec![Data::UnnallocatedTaskPlace(task_id)],
                );
                let pre_allocation_place_id: Uuid = pre_allocation_place.id;
                net.places.insert(pre_allocation_place_id, pre_allocation_place);
                for (agent_id, agent) in self.agents.iter() {

                    let init_place_id = net.query_places(&vec![Data::AgentInitialPlace(*agent_id)], false).first().unwrap().id;

                    let allocation_place = Place::new(
                        format!("{}-alloc", transition.name),
                        TokenSet::Finite,
                        vec![
                            Data::AllocatedTaskPlace(task_id),
                            Data::AgentTaskLockPlace(*agent_id)],
                    );
                    let allocation_place_id = allocation_place.id;
                    net.places.insert(allocation_place_id, allocation_place);


                    // Add a transition from the pre-allocation place to the agent's allocation place
                    let allocation_transition: Transition = Transition {
                        id: Uuid::new_v4(),
                        name: format!("{} decide {}", transition.name, agent.name()),
                        input: vec![(pre_allocation_place_id, 1)].into_iter().collect(),
                        output: vec![(allocation_place_id, 1)].into_iter().collect(),
                        meta_data: vec![
                            Data::AgentTransition(*agent_id),
                            Data::AgentAllocationTransition(*agent_id),
                        ],
                    };
                    net.transitions.insert(allocation_transition.id, allocation_transition);


                    // Add an agent-specific variant of the transition
                    let mut t = transition.clone();
                    t.id = Uuid::new_v4();
                    t.name = format!("{}-{}", agent.name(), t.name);
                    t.input.insert(init_place_id, 1);
                    t.input.insert(allocation_place_id, 1);
                    t.output.insert(init_place_id, 1);
                    t.output.insert(allocation_place_id, 1);
                    t.meta_data.push(Data::AgentTransition(agent.id()));
                    net.transitions.insert(t.id, t);

            }
            }
        }

            // For each transition that isn't a non-agent transition, split the existing transitions into one for each agent.
            // for transition in basic_net.transitions.values() {
            //     if transition.has_data(&vec![Data::NonAgentTransition], true) {
            //         if !non_agent_transitions.contains(&transition.id) {
            //             // Add the non-agent transition to the new network, and add it to the list of non-agent transitions
            //             non_agent_transitions.push(transition.id);
            //             let mut t = transition.clone();
            //             t.id = Uuid::new_v4();
            //             net.transitions.insert(t.id, t);
            //         }
            //     } else {
            //         let pre_allocation_place_id: Uuid;
            //         let allocation_place_id: Uuid;
            //         let task_id = transition.meta_data.iter().find(|d| d.tag() == DataTag::TaskTransition).unwrap().id().unwrap();
            //         if !allocatable_transitions.contains(&transition.id) {
            //             // Add the transition to the list of allocatable transitions
            //             allocatable_transitions.push(transition.id);
            //             let pre_allocation_place = Place::new(
            //                 format!("{}-pre-alloc", transition.name),
            //                 TokenSet::Finite,
            //                 vec![Data::UnnallocatedTaskPlace(task_id)],
            //             );
            //             let allocation_place = Place::new(
            //                 format!("{}-alloc", transition.name),
            //                 TokenSet::Finite,
            //                 vec![Data::AllocatedTaskPlace(task_id),Data::AgentTaskLockPlace(*agent_id)],
            //             );
            //             pre_allocation_place_id = pre_allocation_place.id;
            //             allocation_place_id = allocation_place.id;
            //             net.places.insert(pre_allocation_place.id,pre_allocation_place);
            //             net.places.insert(allocation_place.id,allocation_place);
            //         } else {
            //             pre_allocation_place_id = net.query_places(&vec![Data::UnnallocatedTaskPlace(task_id)], false).first().unwrap().id;
            //             allocation_place_id = net.query_places(&vec![Data::AllocatedTaskPlace(task_id),Data::AgentTaskLockPlace(*agent_id)], false).first().unwrap().id;
            //         }

            //         // Add a transition for each agent from the pre-allocation place
            //         let agent_allocation_transition = Transition {
            //             id: Uuid::new_v4(),
            //             name: format!("{}-alloc-{}", transition.name, agent.name()),
            //             input: vec![(pre_allocation_place_id, 1)].into_iter().collect(),
            //             output: vec![(allocation_place_id, 1)].into_iter().collect(),
            //             meta_data: vec![
            //                 Data::AgentTransition(*agent_id),
            //                 Data::AgentAllocationTransition(*agent_id)
            //             ],
            //         };
            //         net.transitions.insert(agent_allocation_transition.id, agent_allocation_transition);

            //         let mut t = transition.clone();
            //         t.id = Uuid::new_v4();
            //         t.name = format!("{}-{}", agent.name(), t.name);
            //         t.input.insert(init_place_id, 1);
            //         t.output.insert(init_place_id, 1);
            //         t.meta_data.push(Data::AgentTransition(agent.id()));
            //         net.transitions.insert(t.id, t);


            //     }
            // }
        // }

        // For each existing non-agent transition, create a place to function as a lock, and a set of transitions for each agent. 
        // for transition in basic_net.transitions.values() {
        //     if !transition.has_data(&vec![Data::NonAgentTranstion], true) {
        //         let transition_lock_place = Place::new(
        //             format!("{}-lock", transition.name),
        //             TokenSet::Finite,
        //             vec![Data::TaskLock(transition.id)],
        //         );
        //         let lock_id: Uuid = transition_lock_place.id;
        //         let transition_nodes = net.transitions_derived_from_task(
        //             transition
        //                 .meta_data
        //                 .iter()
        //                 .find(|d| d.is_task_transition())
        //                 .unwrap()
        //                 .id()
        //                 .unwrap(),
        //         );
    
        //         for transition_node in transition_nodes {
        //             transition_node.input.insert(lock_id, 1);
        //             transition_node.output.insert(lock_id, 1);
        //         }
        //         net.places.insert(lock_id, transition_lock_place);
        //     }
            
        // }
        net
    }


}