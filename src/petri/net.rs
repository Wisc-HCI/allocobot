use std::collections::HashMap;
use crate::petri::data::Data;
use crate::petri::matrix::MatrixNet;
use crate::petri::place::Place;
use crate::petri::transition::Transition;
use uuid::Uuid;
use nalgebra::{DMatrix};
use colorous::CATEGORY10;
use colorous::TABLEAU10;
use colorous::Color;

pub fn random_agent_color(index: u8) -> Color {
    TABLEAU10[index as usize % TABLEAU10.len()]
}

pub fn random_task_color(index: u8) -> Color {
    CATEGORY10[index as usize % CATEGORY10.len()]
}

pub struct PetriNet {
    pub id: Uuid,
    pub name: String,
    pub places: HashMap<Uuid, Place>,
    pub transitions: HashMap<Uuid, Transition>
}

impl PetriNet {

    pub fn new(name: String) -> Self {
        Self {id:Uuid::new_v4(), name, places:HashMap::new(), transitions:HashMap::new()}
    }
    
    pub fn get_dot(&self) -> String {
        let mut colors: HashMap<Uuid, Color> = HashMap::new();
        let mut agents: u8 = 0;
        let mut tasks: u8 = 0;

        let mut dot = String::from(&format!("digraph {} {{\n", self.name.replace(" ","_")));
        for place in self.places.values() {
            let mut background: Color = Color {r: 255, g: 255, b: 255};
            place.meta_data.iter().for_each(|meta| match meta {
                Data::AgentInitialPlace(id) => {
                    if !colors.contains_key(id) {
                        colors.insert(*id, random_agent_color(agents));
                        agents+=1;
                    }
                    background = colors.get(id).unwrap().clone();
                },
                Data::AgentIndeterminitePlace(id) => {
                    if !colors.contains_key(id) {
                        colors.insert(*id, random_agent_color(agents));
                        agents+=1;
                    }
                    background = colors.get(id).unwrap().clone();
                },
                Data::AgentTaskLockPlace(id) => {
                    if !colors.contains_key(id) {
                        colors.insert(*id, random_agent_color(agents));
                        agents+=1;
                    }
                    background = colors.get(id).unwrap().clone();
                },
                _ => {}
            });
            dot.push_str(&format!("// Place {}\n", place.name));
            dot.push_str(&format!(
                "\t{} [label=\"{}\",style=filled,fillcolor=\"#{:X}\",penwidth=3];\n",
                place.id.as_u128(),
                place.name,
                background
            ));
        }

        for transition in self.transitions.values() {
            let mut font_color: Color = Color {r: 255, g: 255, b: 255};
            let mut border_color: Color = Color {r: 255, g: 255, b: 255};
            transition.meta_data.iter().for_each(|meta| match meta {
                Data::AgentTransition(id) => {
                    if !colors.contains_key(id) {
                        colors.insert(*id, random_agent_color(agents));
                        agents+=1;
                    }
                    font_color = colors.get(id).unwrap().clone();
                }
                Data::TaskTransition(id) => {
                    if !colors.contains_key(id) {
                        colors.insert(*id, random_task_color(tasks));
                        tasks+=1;
                    }
                    border_color = colors.get(id).unwrap().clone();
                }
                _ => {}
            });
            dot.push_str(&format!("// Transition {}\n", transition.name));
            dot.push_str(&format!(
                "\t{} [label=\"{}\",shape=box,style=filled,fillcolor=\"#000000\",fontcolor=\"#{:X}\",color=\"#{:X}\",penwidth=3];\n",
                transition.id.as_u128(),
                transition.name,
                font_color,
                border_color
            ));
        }
        
        for (id, transition) in self.transitions.iter() {
            for (place_id, count) in transition.input.iter() {
                let mut line_color: Color = Color {r: 0, g: 0, b: 0};
                transition.meta_data.iter().for_each(|meta| match meta {
                    Data::AgentTransition(id) => {
                        if !colors.contains_key(id) {
                            colors.insert(*id, random_agent_color(agents));
                            agents+=1;
                        }
                        line_color = colors.get(id).unwrap().clone();
                    }
                    _ => {}
                });
                dot.push_str(&format!(
                    "\t{} -> {} [label=\"{}\",color=\"#{:X}\",penwidth=3];\n",
                    place_id.as_u128(),
                    id.as_u128(),
                    count,
                    line_color
                ));
            }
            for (place_id, count) in transition.output.iter() {
                let mut line_color: Color = Color {r: 0, g: 0, b: 0};
                transition.meta_data.iter().for_each(|meta| match meta {
                    Data::AgentTransition(id) => {
                        if !colors.contains_key(id) {
                            colors.insert(*id, random_agent_color(agents));
                            agents+=1;
                        }
                        line_color = colors.get(id).unwrap().clone();
                    }
                    _ => {}
                });
                dot.push_str(&format!(
                    "\t{} -> {} [label=\"{}\",color=\"#{:X}\",penwidth=3];\n",
                    id.as_u128(),
                    place_id.as_u128(),
                    count,
                    line_color
                ));
            }
        }

        dot.push_str("overlap=false\n");
        dot.push_str("}");
        dot
    }

    pub fn query_transitions(&self, meta_data: &Vec<Data>, fuzzy: bool) -> Vec<&Transition> {
        self.transitions
            .values()
            .filter(|transition| transition.has_data(meta_data, fuzzy))
            .collect()
    }

    pub fn query_places(&self, meta_data: &Vec<Data>, fuzzy: bool) -> Vec<&Place> {
        self.places
            .values()
            .filter(|place| place.has_data(meta_data, fuzzy))
            .collect()
    }

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

    pub fn to_matrix_form(&self) -> MatrixNet {
        let f_v_len: usize = self.places.len() + self.transitions.len() as usize;
        let c_rows: usize = self.places.len() as usize;
        let c_cols: usize = self.transitions.len() as usize;
        let marking_row: usize = self.places.len() as usize;
        let marking_col: usize = 1 as usize;

        // let mut f: DMatrix<u128, f_v_len, f_v_len> = DMatrix::from_iterator((1..).map(|x| 0 as u128));
        // let mut v: DMatrix<u128, f_v_len, f_v_len> = DMatrix::from_iterator((1..).map(|x| 0 as u128));
        // let mut c: DMatrix<i128, c_rows, c_cols> = DMatrix::from_iterator((1..).map(|x| 0 as i128));
        // let mut m: DMatrix<u128, marking_row, marking_col> = DMatrix::from_iterator((1..).map(|x| 0 as u128));
        let mut matrix = MatrixNet {
            id: Uuid::new_v4(),
            name: self.name.clone(),
            f: DMatrix::from_element(f_v_len, f_v_len, 0 as u128),
            v: DMatrix::from_element(f_v_len, f_v_len, 0 as u128),
            c: DMatrix::from_element(c_rows, c_cols, 0 as i128),
            marking: DMatrix::from_element(marking_row, marking_col, 0 as u128)
        };

        
        let mut cur_row: usize = 0;
        let mut cur_col: usize = self.places.len();
        // let mut _c_cur_row: usize = 0;
        // let mut _c_cur_col: usize = 0;
        let mut delta: usize;
        
        // Fill in C and the top half of F and V
        for (p_key, _p_val) in self.places.iter() {
            // TODO: fill marking
            // matrix.marking[(cur_row, 0)] = self.places[p_key].tokens.length();
            for (_t_key, t_val) in self.transitions.iter() {
                delta = 0;
                for (i_key, i_val) in t_val.input.iter() {
                    if i_key.eq(&p_key) {
                        matrix.f[(cur_row, cur_col)] = 1;
                        matrix.v[(cur_row, cur_col)] = t_val.input[i_key] as u128;
                        delta += i_val;
                    }
                }
                for (i_key, i_val) in t_val.output.iter() {
                    if i_key.eq(&p_key) {
                        delta -= i_val;
                    }
                }
                matrix.c[(cur_row, cur_col)] = delta as i128;
                cur_col += 1;
                // c_cur_col += 1;
            }

            cur_col = self.places.len();
            cur_row += 1;
            // c_cur_row += 1;
            // c_cur_col = 0;
        }

        cur_col = 0;
        // Bottom half (of F and V)
        for (_t_key, t_val) in self.transitions.iter() {
            for (p_key, _p_val) in self.places.iter() {
                for (i_key, _i_val) in t_val.output.iter() {
                    if i_key.eq(&p_key) {
                        matrix.f[(cur_row, cur_col)] = 1;
                        matrix.v[(cur_row, cur_col)] = t_val.output[i_key] as u128;
                    }
                }

                cur_col += 1
            }

            cur_col = 0;
            cur_row += 1;
        }

        return matrix;
    }
}
