use std::collections::HashMap;
use crate::petri::data::Data;
use crate::petri::matrix::MatrixNet;
use crate::petri::place::Place;
use crate::petri::transition::Transition;
use crate::petri::token::TokenSet;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use nalgebra::{DMatrix};
use colorous::CATEGORY10;
use colorous::TABLEAU10;
use colorous::Color;
use rand::prelude::*;
use rand::seq::SliceRandom;
use std::f64::MIN;

pub fn random_agent_color(index: u8) -> Color {
    TABLEAU10[index as usize % TABLEAU10.len()]
}

pub fn random_task_color(index: u8) -> Color {
    CATEGORY10[index as usize % CATEGORY10.len()]
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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


        let mut matrix = MatrixNet {
            id: Uuid::new_v4(),
            name: self.name.clone(),
            arcs: DMatrix::from_element(f_v_len, f_v_len, 0 as i64),
            weights: DMatrix::from_element(f_v_len, f_v_len, 0 as i64),
            incidence: DMatrix::from_element(c_rows, c_cols, 0 as i64),
            marking: DMatrix::from_element(marking_row, marking_col, 0 as i64)
        };

        let mut cur_row: usize = 0;
        let mut cur_col: usize = self.places.len();
        let mut incidence_col: usize = 0;
        let mut delta: i64;
        
        // Fill in C and the top half of F and V
        for (p_key, p_val) in self.places.iter() {
            // TODO: better way of getting the initial marking?
            // Mark the sources as having a token initially
            if p_val.tokens == TokenSet::Infinite {
                matrix.marking[(cur_row, 0)] = 1;
            }


            // For every place, transition pair mark whether an arc is present, the weight, and incidence
            for (_t_key, t_val) in self.transitions.iter() {
                delta = 0;
                for (i_key, i_val) in t_val.input.iter() {
                    if i_key.eq(&p_key) {
                        matrix.arcs[(cur_row, cur_col)] = 1;
                        matrix.weights[(cur_row, cur_col)] = t_val.input[i_key] as i64;

                        // If the place is a source, then it doesn't reduce the number of tokens since it's infinite
                        if p_val.tokens != TokenSet::Infinite {
                            delta -= *i_val as i64;
                        }
                    }
                }
                for (i_key, i_val) in t_val.output.iter() {
                    if i_key.eq(&p_key) {
                        delta += *i_val as i64;
                    }
                }
                matrix.incidence[(cur_row, incidence_col)] = delta;
                cur_col += 1;
                incidence_col += 1;
            }

            cur_col = self.places.len();
            cur_row += 1;
            incidence_col = 0;
        }

        cur_col = 0;
        // Bottom half (of F and V)
        for (_t_key, t_val) in self.transitions.iter() {
            for (p_key, _p_val) in self.places.iter() {
                for (i_key, _i_val) in t_val.output.iter() {
                    if i_key.eq(&p_key) {
                        matrix.arcs[(cur_row, cur_col)] = 1;
                        matrix.weights[(cur_row, cur_col)] = t_val.output[i_key] as i64;
                    }
                }

                cur_col += 1
            }

            cur_col = 0;
            cur_row += 1;
        }

        return matrix;
    }

    fn reward_value(&self, previous_state: DMatrix<i64>, new_state: DMatrix<i64>, goal_state: DMatrix<i64>) -> f64 {
        if new_state.eq(&previous_state) {
            return 0.0;
        } 
        let (rows, _cols) = new_state.shape();
        let mut in_goal_state = false;
        for i in 0..rows {
            if new_state[(i, 0)] - previous_state[(i, 0)] == goal_state[(i, 0)] {
                in_goal_state = true;
            }
        }

        if in_goal_state {
            return 999.0;
        }

        return -5.0;
    }

    pub fn q_learning(&self) -> HashMap<DMatrix<i64>, HashMap<i64, f64>> {
        let matrix = self.to_matrix_form();

        let mut q_matrix: HashMap<DMatrix<i64>, HashMap<i64, f64>> = HashMap::new();
        
        let max_retries = 100;
        let max_steps = 100;

        let alpha = 0.2;
        let gamma = 0.8;

        let number_of_actions = self.transitions.len();
        let marking_row = self.places.len();

        // TODO: better goal setup?
        let mut goal_state = DMatrix::from_element(marking_row, 1, 0 as i64);
        
        let mut goal_row = 0;
        for (_p_key, p_val) in self.places.iter() {
            // TODO: better way of getting the initial marking?
            // Mark the sources as having a token initially
            if p_val.tokens == TokenSet::Sink {
                goal_state[(goal_row, 0)] = 1;
            }
            goal_row += 1;
        }

        for _i in 0..max_retries {
            // Reset marking to orginal after each retry
            let mut marking = matrix.marking.clone();

            for _j in 0..max_steps {
                let mut max_value = MIN;
                let mut max_options: Vec<DMatrix<i64>> = Vec::new();

                // let mut action_count = 0;
                for action_count in 0..number_of_actions {
                    let mut action: DMatrix<i64> = DMatrix::from_element(number_of_actions, 1, 0 as i64);
                    action[(action_count, 0)] = 1;
                    let mut marking_prime = DMatrix::from_element(marking_row as usize, 1, 0 as i64);
                    let tmp = matrix.incidence.clone() * &action;
                    let mut is_valid_action = true;

                    // Calculate the new marking and whether the action is valid
                    for k in 0..marking_row {
                        marking_prime[(k, 0)] = tmp[(k, 0)] + marking[(k, 0)];
                        if marking_prime[(k, 0)] < 0 {
                            is_valid_action = false;
                        }
                    }
                    
                    // Action is not valid, as that drops tokens below 0 in some place
                    if !is_valid_action {
                        continue;
                    }

                    // Ensure that the q_matrix has reference of the current marking and action
                    match q_matrix.get(&marking) {
                        Some(hshmap) => {
                            match hshmap.get(&(action_count as i64)) {
                                Some(_value) => (),
                                None => {
                                    let mut tmp_hash_map = hshmap.clone();
                                    tmp_hash_map.insert(action_count as i64, 0.0);
                                    q_matrix.insert(marking.clone(), tmp_hash_map);
                                }
                            };
                        },
                        None => {
                            let mut tmp_hash_map = HashMap::new();
                            tmp_hash_map.insert(action_count as i64, 0.0);
                            q_matrix.insert(marking.clone(), tmp_hash_map);
                        }
                    };


                    let mut max_action = MIN;
                    for action_prime_count in 0..number_of_actions {
                        let mut action_prime = DMatrix::from_element(number_of_actions, 1, 0 as i64);
                        action_prime[(action_prime_count, 0)] = 1;
                        let mut marking_double_prime = DMatrix::from_element(marking_row as usize, 1, 0 as i64);
                        // let tmp_prime = matrix.incidence.clone().ad_mul(&action_prime);
                        let tmp_prime = matrix.incidence.clone() * &action_prime;
                        let mut is_valid_action_prime = true;

                        // Calculate the new marking and whether the action is valid
                        for k in 0..marking_row {
                            marking_double_prime[(k, 0)] = tmp_prime[(k, 0)] + marking_prime[(k, 0)];
                            if marking_double_prime[(k, 0)] < 0 {
                                is_valid_action_prime = false;
                            }
                        }

                        // Action is not valid, as that drops tokens below 0 in some place
                        if !is_valid_action_prime {
                            continue;
                        }

                        // Ensure that the q_matrix has reference of potential next marking and potential next action
                        let mut current_q_prime_value = MIN;
                        match q_matrix.get(&marking_prime) {
                            Some(hshmap) => {
                                match hshmap.get(&(action_prime_count as i64)) {
                                    Some(value) => current_q_prime_value = value.clone(),
                                    None => {
                                        let mut tmp_hash_map = hshmap.clone();
                                        tmp_hash_map.insert(action_prime_count as i64, 0.0);
                                        q_matrix.insert(marking_prime.clone(), tmp_hash_map);
                                    }
                                };
                            },
                            None => {
                                let mut tmp_hash_map = HashMap::new();
                                tmp_hash_map.insert(action_prime_count as i64, 0.0);
                                q_matrix.insert(marking_prime.clone(), tmp_hash_map);
                            }
                        };
                        
                        if current_q_prime_value > max_action {
                            max_action = current_q_prime_value;
                        }
                    }
                    let mut new_value = MIN;
                    let reward = self.reward_value(marking.clone(), marking_prime.clone(), goal_state.clone());
                    match q_matrix.get(&marking) {
                        Some(hshmap) => {
                            match hshmap.get(&(action_count as i64)) {
                                Some(value) => {
                                    new_value = (1.0 - alpha) * value.clone() + alpha * (reward + gamma * max_action);
                                    let mut tmp_hash_map = hshmap.clone();
                                    tmp_hash_map.insert(action_count as i64, new_value.clone());
                                    q_matrix.insert(marking.clone(), tmp_hash_map);
                                },
                                None => ()
                            };
                        },
                        None => ()
                    }
                    if new_value > max_value {
                        max_value = new_value.clone();
                        max_options = Vec::new();
                        max_options.push(marking_prime.clone());
                    } else if new_value == max_value {
                        max_options.push(marking_prime.clone());
                    }
                }
                let choose_result = max_options.choose(&mut rand::thread_rng());
                match choose_result {
                    Some(x) => marking = x.clone(),
                    None => ()
                };

            }
        }

        return q_matrix;
    }
}
