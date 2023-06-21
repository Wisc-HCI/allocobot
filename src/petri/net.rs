use std::collections::HashMap;
use std::error::Error;

use crate::description::target;
use crate::description::task::Task;
// use crate::description::target::Target;
use crate::petri::place::Place;
use crate::petri::transition::Transition;
use crate::petri::token::TokenSet;
use crate::petri::matrix::MatrixNet;
use uuid::Uuid;
use std::fmt;
use nalgebra::base::DMatrix;
use nalgebra::base::ArrayStorage;

trait PetriNet<'a> {
    fn get_places(&mut self) -> HashMap<Uuid, &mut Place>;
    fn get_transitions(&mut self) -> HashMap<Uuid, &mut Transition>;
    fn get_tasks(&mut self) -> HashMap<Uuid, &mut Task<'a>>;
}

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
        let mut places: HashMap<Uuid, Place> = HashMap::new();
        let mut transitions: HashMap<Uuid, Transition> = HashMap::new();
        let mut net: BasicNet = Self::new(name);
        let mut added: HashMap<Uuid, Task<'a>> = HashMap::new();

        // Add all tasks as places to the net
        for task in tasks.iter() {
            match task {
                Task::Process(process) => {
                    for target in process.output.iter() {
                        let place = Place::new(
                            format!("place-{}-{:?}", target.0.name, process.name), 
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
                            format!("place-{}-{:?}", target.0.name, spawn.name), 
                            TokenSet::Infinite, 
                            Some(task.id()),
                            Some(target.0.id),
                        );
                        net.places.insert(place.id, place);
                    }
                }
                Task::Complete(complete) => {
                    let place = Place::new(
                        format!("place-{:?}", complete.name), 
                        TokenSet::Sink, 
                        Some(task.id()),
                        None
                    );
                    net.places.insert(place.id, place);
                }
            }
        }

        // Add all dependencies as transitions to the net
        for task in tasks.iter() {
            let mut input: HashMap<Uuid, usize> = HashMap::new();
            let mut output: HashMap<Uuid, usize> = HashMap::new();
            for dependency in task.dependencies() {
                let dep_task = tasks.iter().find(|t| t.id()==dependency.task.id()).unwrap();
                let matching_dep_places = net.query_places(Some(dependency.task.id()), Some(dependency.target.id));
                if !matching_dep_places.is_empty() {
                    let place = matching_dep_places[0];
                    if place.tokens == TokenSet::Infinite {
                        input.insert(place.id, dependency.count);
                    } else if place.tokens == TokenSet::Finite && place.source_task.map_or_else(|| false, |t| {
                        dep_task.output_target_count(&dependency.target.id) >= dependency.count
                    }) {
                        input.insert(place.id, dependency.count);
                    } else {
                        return Err(format!("Error Building Basic Net: Dependency for task {} cannot be satisfied. Task {}, which is a dependency, only produces {} of {} ({} needed)", 
                            task.name(), 
                            dependency.task.name(), 
                            dependency.task.output_target_count(&dependency.target.id), 
                            dependency.target.name, 
                            dependency.count
                        ));
                    }
                } else {
                    return Err(format!("Error Building Basic Net: Dependency for task {} cannot be satisfied. Task {} cannot be found", 
                            task.name(), 
                            dependency.task.name()
                        ));
                }
            }
            for (target,count) in task.output() {
                let matching_places = net.query_places(Some(task.id()), Some(target.id));
                match matching_places.first() {
                    Some(place) => {
                        output.insert(place.id, count);
                    },
                    None => {
                        return Err(format!("Error Building Basic Net: Output for task {} cannot be satisfied. Task {} cannot be found", 
                                task.name(), 
                                target.name
                            ));
                    }
                }
            }
            let transition: Transition = Transition {
                id: Uuid::new_v4(),
                name: format!("transition-{:?}", task.name()),
                input,
                output,
                source_task: Some(task.id()),
            };

            net.transitions.insert(transition.id, transition);
        }

                // let transition: Transition = Transition::new(format!("transition-{:?}", dependency.id()), Some(task.id()))
                //     .with_input(&dependency.task.id(), 1)
                //     .with_output(&task.id(), 1);
                // net.transitions.insert(transition.id, transition);

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

    pub fn to_matrix_form(&self) -> MatrixNet {
        let fVlen: usize = self.places.len() + self.transitions.len() as usize;
        let cRows: usize = self.places.len() as usize;
        let cCols: usize = self.transitions.len() as usize;
        let markingRow: usize = self.places.len() as usize;
        let markingCol: usize = 1 as usize;

        // let mut f: DMatrix<u128, fVlen, fVlen> = DMatrix::from_iterator((1..).map(|x| 0 as u128));
        // let mut v: DMatrix<u128, fVlen, fVlen> = DMatrix::from_iterator((1..).map(|x| 0 as u128));
        // let mut c: DMatrix<i128, cRows, cCols> = DMatrix::from_iterator((1..).map(|x| 0 as i128));
        // let mut m: DMatrix<u128, markingRow, markingCol> = DMatrix::from_iterator((1..).map(|x| 0 as u128));
        let mut matrix = MatrixNet {
            id: Uuid::new_v4(),
            name: self.name.clone(),
            F: DMatrix::from_element(fVlen, fVlen, 0 as u128),
            V: DMatrix::from_element(fVlen, fVlen, 0 as u128),
            C: DMatrix::from_element(cRows, cCols, 0 as i128),
            marking: DMatrix::from_element(markingRow, markingCol, 0 as u128)
        };

        
        let mut curRow: usize = 0;
        let mut curCol: usize = self.places.len();
        let mut cCurRow: usize = 0;
        let mut cCurCol: usize = 0;
        let mut delta = 0;
        
        // Fill in C and the top half of F and V
        for (pKey, pVal) in self.places.iter() {
            // TODO: fill marking
            // matrix.marking[(curRow, 0)] = self.places[pKey].tokens.length();
            for (tKey, tVal) in self.transitions.iter() {
                delta = 0;
                for (iKey, iVal) in tVal.input.iter() {
                    if iKey.eq(&pKey) {
                        matrix.F[(curRow, curCol)] = 1;
                        matrix.V[(curRow, curCol)] = tVal.input[iKey] as u128;
                        delta += iVal;
                    }
                }
                for (iKey, iVal) in tVal.output.iter() {
                    if iKey.eq(&pKey) {
                        delta -= iVal;
                    }
                }
                matrix.C[(curRow, curCol)] = delta as i128;
                curCol += 1;
                cCurCol += 1;
            }

            curCol = self.places.len();
            curRow += 1;
            cCurRow += 1;
            cCurCol = 0;
        }

        curCol = 0;
        // Bottom half (of F and V)
        for (tKey, tVal) in self.transitions.iter() {
            for (pKey, pVal) in self.places.iter() {
                for (iKey, iVal) in tVal.output.iter() {
                    if iKey.eq(&pKey) {
                        matrix.F[(curRow, curCol)] = 1;
                        matrix.V[(curRow, curCol)] = tVal.output[iKey] as u128;
                    }
                }

                curCol += 1
            }

            curCol = 0;
            curRow += 1;
        }

        return matrix;
    }
}

impl <'a> PetriNet<'a> for BasicNet<'a> {
    fn get_places(&mut self) -> HashMap<Uuid, &mut Place> {
        let mut places: HashMap<Uuid, &mut Place> = HashMap::new();
        self.places.values_mut().for_each(|place: &mut Place| {
            places.insert(place.id, place);
        });
        places
    }

    fn get_transitions(&mut self) -> HashMap<Uuid, &mut Transition> {
        let mut transitions: HashMap<Uuid, &mut Transition> = HashMap::new();
        self.transitions.values_mut().for_each(|transition: &mut Transition| {
            transitions.insert(transition.id, transition);
        });
        transitions
    }

    fn get_tasks(&mut self) -> HashMap<Uuid, &mut Task<'a>> {
        let mut tasks: HashMap<Uuid, &mut Task> = HashMap::new();
        self.tasks.values_mut().for_each(|task: &mut Task| {
            tasks.insert(task.id(), task);
        });
        tasks
    }
}

impl <'a> fmt::Display for BasicNet<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BasicNet {} ({}): {{\n", self.name, self.id)?;
        if self.places.is_empty() {
            write!(f, "\tPlaces: [],\n")?;
        } else {
            write!(f, "\tPlaces: [\n")?;
            for place in self.places.values() {
                write!(f, "\t\t{}: {{ name: {}, tokens: {:?}, source_task: {:?}}},\n", place.id, place.name, place.tokens, place.source_task)?;
            }
            write!(f, "\t],\n")?;
        }

        if self.transitions.is_empty() {
            write!(f, "\tTransitions: [],\n")?;
        } else {
            write!(f, "\tTransitions: [\n")?;
            for transition in self.transitions.values() {
                write!(f, "\t\t{}: {{ name: {}, input: {:?}, output: {:?}, source_task: {:?}}},\n", transition.id, transition.name, transition.input,  transition.output, transition.source_task)?;
            }
            write!(f, "\t],\n")?;
        }
       
        write!(f, "}}\n")
    }
}