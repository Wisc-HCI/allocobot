use std::collections::HashMap;
// use std::error::Error;
use crate::petri::data::Data;
// use crate::description::target;
// use crate::description::task::Task;
// use crate::description::target::Target;
use crate::petri::matrix::MatrixNet;
use crate::petri::place::Place;
use crate::petri::transition::Transition;
use uuid::Uuid;
// use std::fmt;
use nalgebra::{Vector3, DMatrix};
use rand::prelude::*;

pub fn color_to_hex(color: (u8, u8, u8)) -> String {
    format!("#{:02x}{:02x}{:02x}", color.0, color.1, color.2)
}

pub fn random_color() -> (u8, u8, u8) {
    let mut rng = rand::thread_rng();
    normalize_color((rng.gen(), rng.gen(), rng.gen()))
}

pub fn normalize_color(color: (u8, u8, u8)) -> (u8, u8, u8) {
    let color_vec: Vector3<f32> = Vector3::new(color.0 as f32, color.1 as f32, color.2 as f32);
    let normalized = color_vec.normalize();
    // Bump up the brightness by multiplying by 280 instead of 255
    (
        (normalized.x * 280.0) as u8,
        (normalized.y * 280.0) as u8,
        (normalized.z * 280.0) as u8,
    )
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
        let mut colors: HashMap<Uuid, (u8, u8, u8)> = HashMap::new();

        let mut dot = String::from(&format!("digraph {} {{\n", self.name.replace(" ","_")));
        for place in self.places.values() {
            let mut background: (u8, u8, u8) = (255, 255, 255);
            place.meta_data.iter().for_each(|meta| match meta {
                Data::AgentInitial(id) => {
                    if !colors.contains_key(id) {
                        colors.insert(*id, random_color());
                    }
                    background = colors.get(id).unwrap().clone();
                }
                _ => {}
            });
            dot.push_str(&format!("// Place {}\n", place.name));
            dot.push_str(&format!(
                "\t{} [label=\"{}\",style=filled,fillcolor=\"{}\",penwidth=3];\n",
                place.id.as_u128(),
                place.name,
                color_to_hex(background)
            ));
        }

        for transition in self.transitions.values() {
            let mut font_color: (u8, u8, u8) = (255, 255, 255);
            let mut border_color: (u8, u8, u8) = (255, 255, 255);
            transition.meta_data.iter().for_each(|meta| match meta {
                Data::AgentTransition(id) => {
                    if !colors.contains_key(id) {
                        colors.insert(*id, random_color());
                    }
                    font_color = colors.get(id).unwrap().clone();
                }
                Data::TaskTransition(id) => {
                    if !colors.contains_key(id) {
                        colors.insert(*id, random_color());
                    }
                    border_color = colors.get(id).unwrap().clone();
                }
                _ => {}
            });
            dot.push_str(&format!("// Transition {}\n", transition.name));
            dot.push_str(&format!(
                "\t{} [label=\"{}\",shape=box,style=filled,fillcolor=\"#000000\",fontcolor=\"{}\",color=\"{}\",penwidth=3];\n",
                transition.id.as_u128(),
                transition.name,
                color_to_hex(font_color),
                color_to_hex(border_color)
            ));
        }
        
        for (id, transition) in self.transitions.iter() {
            for (place_id, count) in transition.input.iter() {
                let mut line_color: (u8, u8, u8) = (0, 0, 0);
                transition.meta_data.iter().for_each(|meta| match meta {
                    Data::AgentTransition(id) => {
                        if !colors.contains_key(id) {
                            colors.insert(*id, random_color());
                        }
                        line_color = colors.get(id).unwrap().clone();
                    }
                    _ => {}
                });
                dot.push_str(&format!(
                    "\t{} -> {} [label=\"{}\",color=\"{}\",penwidth=3];\n",
                    place_id.as_u128(),
                    id.as_u128(),
                    count,
                    color_to_hex(line_color)
                ));
            }
            for (place_id, count) in transition.output.iter() {
                let mut line_color: (u8, u8, u8) = (0, 0, 0);
                transition.meta_data.iter().for_each(|meta| match meta {
                    Data::AgentTransition(id) => {
                        if !colors.contains_key(id) {
                            colors.insert(*id, random_color());
                        }
                        line_color = colors.get(id).unwrap().clone();
                    }
                    _ => {}
                });
                dot.push_str(&format!(
                    "\t{} -> {} [label=\"{}\",color=\"{}\",penwidth=3];\n",
                    id.as_u128(),
                    place_id.as_u128(),
                    count,
                    color_to_hex(line_color)
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
