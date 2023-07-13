use crate::petri::data::{Data, Query};
#[cfg(test)]
use crate::petri::data::DataTag;
use crate::petri::matrix::MatrixNet;
use crate::petri::place::Place;
use crate::petri::token::TokenSet;
use crate::petri::transition::Transition;
use colorous::Color;
use colorous::SET3;
use colorous::TABLEAU10;
use nalgebra::DMatrix;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::MIN;
use uuid::Uuid;
use enum_tag::EnumTag;

pub fn random_agent_color(index: u8) -> Color {
    TABLEAU10[index as usize % TABLEAU10.len()]
}

pub fn random_task_color(index: u8) -> Color {
    SET3[index as usize % SET3.len()]
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PetriNet {
    pub id: Uuid,
    pub name: String,
    pub places: HashMap<Uuid, Place>,
    pub transitions: HashMap<Uuid, Transition>,
    pub initial_marking: HashMap<Uuid, usize>,
    pub name_lookup: HashMap<Uuid, String>,
}

impl PetriNet {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            places: HashMap::new(),
            transitions: HashMap::new(),
            initial_marking: HashMap::new(),
            name_lookup: HashMap::new(),
        }
    }

    pub fn get_dot(&self) -> String {
        let mut colors: HashMap<Uuid, Color> = HashMap::new();
        let mut agents: u8 = 0;
        let mut tasks: u8 = 0;

        let mut dot = String::from(&format!(
            "digraph {} {{\nbgcolor=\"transparent\"\n",
            self.name.replace(" ", "_")
        ));
        for place in self.places.values() {
            let mut background: Color = Color {
                r: 255,
                g: 255,
                b: 255,
            };
            let mut border_color: Color = Color { r: 0, g: 0, b: 0 };
            let mark: String;
            match place.tokens {
                TokenSet::Infinite => mark = "∞".to_string(),
                TokenSet::Finite => {
                    mark = self
                        .initial_marking
                        .get(&place.id)
                        .unwrap_or(&0)
                        .to_string()
                }
                TokenSet::Sink => mark = "sink".to_string(),
            }
            place.meta_data.iter().for_each(|meta| match meta {
                Data::Agent(id) => {
                    if !colors.contains_key(id) {
                        colors.insert(*id, random_agent_color(agents));
                        agents += 1;
                    }
                    background = colors.get(id).unwrap().clone();
                }
                Data::Task(id) => {
                    if !colors.contains_key(id) {
                        colors.insert(*id, random_task_color(tasks));
                        tasks += 1;
                    }
                    border_color = colors.get(id).unwrap().clone();
                }
                _ => {}
            });
            dot.push_str(&format!("// Place {}\n", place.name));
            dot.push_str(&format!(
                "\t{} [label=\"{}\\n({})\",tooltip=\"{}\",style=filled,fillcolor=\"#{:X}\",color=\"#{:X}\",penwidth=3];\n",
                place.id.as_u128(),
                place.name,
                mark,
                place.meta_data.iter().map(|meta| {
                    self.data_to_label(meta)
                }).collect::<Vec<String>>().join("\\n"),
                background,
                border_color,
            ));
        }

        for transition in self.transitions.values() {
            let mut font_color: Color = Color {
                r: 255,
                g: 255,
                b: 255,
            };
            let mut border_color: Color = Color { r: 0, g: 0, b: 0 };
            transition.meta_data.iter().for_each(|meta| match meta {
                Data::Agent(id) => {
                    if !colors.contains_key(id) {
                        colors.insert(*id, random_agent_color(agents));
                        agents += 1;
                    }
                    font_color = colors.get(id).unwrap().clone();
                }
                Data::Task(id) => {
                    if !colors.contains_key(id) {
                        colors.insert(*id, random_task_color(tasks));
                        tasks += 1;
                    }
                    border_color = colors.get(id).unwrap().clone();
                }
                _ => {}
            });
            dot.push_str(&format!("// Transition {}\n", transition.name));
            dot.push_str(&format!(
                "\t{} [label=\"{}\",tooltip=\"{}\",shape=box,style=filled,fillcolor=\"#000000\",fontcolor=\"#{:X}\",color=\"#{:X}\",penwidth=3];\n",
                transition.id.as_u128(),
                transition.name,
                transition.meta_data.iter().map(|meta| {
                    self.data_to_label(meta)
                }).collect::<Vec<String>>().join("\\n"),
                font_color,
                border_color
            ));
        }

        for (id, transition) in self.transitions.iter() {
            for (place_id, count) in transition.input.iter() {
                let mut line_color: Color = Color {
                    r: 100,
                    g: 100,
                    b: 100,
                };
                transition.meta_data.iter().for_each(|meta| match meta {
                    Data::Agent(id) => {
                        if !colors.contains_key(id) {
                            colors.insert(*id, random_agent_color(agents));
                            agents += 1;
                        }
                        line_color = colors.get(id).unwrap().clone();
                    }
                    _ => {}
                });
                dot.push_str(&format!(
                    "\t{} -> {} [label=\"{}\",color=\"#{:X}\",fontcolor=\"#{:X}\",penwidth=3];\n",
                    place_id.as_u128(),
                    id.as_u128(),
                    count,
                    line_color,
                    line_color
                ));
            }
            for (place_id, count) in transition.output.iter() {
                let mut line_color: Color = Color {
                    r: 100,
                    g: 100,
                    b: 100,
                };
                transition.meta_data.iter().for_each(|meta| match meta {
                    Data::Agent(id) => {
                        if !colors.contains_key(id) {
                            colors.insert(*id, random_agent_color(agents));
                            agents += 1;
                        }
                        line_color = colors.get(id).unwrap().clone();
                    }
                    _ => {}
                });
                dot.push_str(&format!(
                    "\t{} -> {} [label=\"{}\",color=\"#{:X}\",fontcolor=\"#{:X}\",penwidth=3];\n",
                    id.as_u128(),
                    place_id.as_u128(),
                    count,
                    line_color,
                    line_color
                ));
            }
        }

        dot.push_str("overlap=false\n");
        dot.push_str("}");
        dot
    }

    pub fn data_to_label(&self, data: &Data) -> String {
        match data.id() {
            None => return format!("{:?}",data.tag()),
            Some(id) => return format!("{:?}:{}",data.tag(),self.name_lookup.get(&id).unwrap_or(&"Unknown".to_string()))
        }
    }

    pub fn query_transitions(&self, query_vec: &Vec<Query>) -> Vec<&Transition> {
        self.transitions
            .values()
            .filter(|transition| transition.has_data(query_vec))
            .collect()
    }

    pub fn query_places(&self, query_vec: &Vec<Query>) -> Vec<&Place> {
        self.places
            .values()
            .filter(|place| place.has_data(query_vec))
            .collect()
    }

    pub fn delete_transition(&mut self, id: Uuid) {
        if self.transitions.contains_key(&id) {
            self.transitions.remove(&id);
        } else {
            println!("Transition with id {} does not exist", id);
        }
    }

    pub fn delete_place(&mut self, id: Uuid) {
        if self.places.contains_key(&id) {
            self.places.remove(&id);
            self.transitions.iter_mut().for_each(|(_, transition)| {
                transition.input.remove(&id);
                transition.output.remove(&id);
            });
        } else {
            println!("Place with id {} does not exist", id);
        }
    }

    pub fn add_transition_with_edge_conditions<FnIn, FnOut>(
        &mut self,
        name: String,
        meta_data: Vec<Data>,
        input_validator: FnIn,
        output_validator: FnOut,
        input_count: usize,
        output_count: usize,
    ) where
        FnIn: Fn(&Place) -> bool,
        FnOut: Fn(&Place) -> bool,
    {
        let input: HashMap<Uuid, usize> = self
            .places
            .iter()
            .filter(|(_, place)| input_validator(place))
            .map(|(id, _)| (*id, input_count))
            .collect();
        let output: HashMap<Uuid, usize> = self
            .places
            .iter()
            .filter(|(_, place)| output_validator(place))
            .map(|(id, _)| (*id, output_count))
            .collect();
        let transition = Transition {
            id: Uuid::new_v4(),
            name,
            meta_data,
            input,
            output,
        };
        self.transitions.insert(transition.id, transition);
    }

    pub fn split_place<EvalFn>(
        &mut self, 
        id: &Uuid, 
        splits: Vec<Vec<Data>>, 
        eval_fn: EvalFn
    ) -> (Vec<Uuid>,Vec<Uuid>) where 
        EvalFn: Fn(&Transition,&Vec<Data>) -> bool 
    {
        let mut new_places: Vec<Uuid> = Vec::new();
        let mut new_transitions: Vec<Uuid> = Vec::new();
        if self.places.contains_key(id) {
            let template_place = self.places.get(id).unwrap().clone();
            let transition_neighbors: Vec<Uuid> = self
                .transitions
                .iter()
                .filter(|(_, transition)| {
                    transition.input.contains_key(id) || transition.output.contains_key(id)
                })
                .map(|(id, _)| *id)
                .collect();
            let mut removed_transitions: Vec<Uuid> = Vec::new(); 
            for split in splits {
                // println!("Splitting place {:?} for {:?}", id, split);
                let mut new_place = template_place.clone();
                let new_place_id = Uuid::new_v4();
                new_place.id = new_place_id;
                for split_data in split.iter() {
                    new_place.meta_data.push(split_data.clone());
                }
                self.places.insert(new_place_id, new_place);
                new_places.push(new_place_id);

                for transition_id in transition_neighbors.iter() {
                    let existing_transition = self.transitions.get(transition_id).unwrap();
                    // println!("Checking transition {:?}", existing_transition);
                    // println!("Eval fn: {:?}", eval_fn(existing_transition,&split));
                    if eval_fn(existing_transition,&split) {
                        let mut new_transition = existing_transition.clone();
                        new_transition.id = Uuid::new_v4();
                        let new_transition_id = new_transition.id.clone();
                        if new_transition.input.contains_key(id) {
                            new_transition.input.remove(id);
                            new_transition.input.insert(new_place_id, 1);
                        }
                        if new_transition.output.contains_key(id) {
                            new_transition.output.remove(id);
                            new_transition.output.insert(new_place_id, 1);
                        }
                        for split_data in split.iter() {
                            if !new_transition.meta_data.contains(split_data) {
                                new_transition.meta_data.push(split_data.clone());
                            }
                        }
                        self.transitions.insert(new_transition.id, new_transition);
                        new_transitions.push(new_transition_id);
                        if !removed_transitions.contains(transition_id) {
                            removed_transitions.push(*transition_id);
                        }
                    } 
                    if !removed_transitions.contains(transition_id) {
                        removed_transitions.push(*transition_id);
                    }
                }
            }
            for transition_id in removed_transitions.iter() {
                self.transitions.remove(transition_id);
            }
            self.places.remove(id);
        }

        (new_places,new_transitions)
    }

    pub fn transitions_derived_from_task(&mut self, task: Uuid) -> Vec<&mut Transition> {
        self.transitions
            .values_mut()
            .into_iter()
            .filter(|transition| transition.meta_data.contains(&Data::Task(task)))
            .collect()
    }

    pub fn transitions_associated_with_agent(&mut self, agent: Uuid) -> Vec<&mut Transition> {
        self.transitions
            .values_mut()
            .into_iter()
            .filter(|transition| transition.meta_data.contains(&Data::Agent(agent)))
            .collect()
    }

    pub fn transitions_connected_to_place_mut(&mut self, place: Uuid) -> Vec<&mut Transition> {
        self.transitions
            .values_mut()
            .into_iter()
            .filter(|transition| {
                transition.input.contains_key(&place) || transition.output.contains_key(&place)
            })
            .collect()
    }

    pub fn transitions_connected_to_place(&mut self, place: Uuid) -> Vec<&Transition> {
        self.transitions
            .values()
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
            marking: DMatrix::from_element(marking_row, marking_col, 0 as i64),
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

    fn reward_value(
        &self,
        previous_state: DMatrix<i64>,
        new_state: DMatrix<i64>,
        goal_state: DMatrix<i64>,
    ) -> f64 {
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
                    let mut action: DMatrix<i64> =
                        DMatrix::from_element(number_of_actions, 1, 0 as i64);
                    action[(action_count, 0)] = 1;
                    let mut marking_prime =
                        DMatrix::from_element(marking_row as usize, 1, 0 as i64);
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
                        }
                        None => {
                            let mut tmp_hash_map = HashMap::new();
                            tmp_hash_map.insert(action_count as i64, 0.0);
                            q_matrix.insert(marking.clone(), tmp_hash_map);
                        }
                    };

                    let mut max_action = MIN;
                    for action_prime_count in 0..number_of_actions {
                        let mut action_prime =
                            DMatrix::from_element(number_of_actions, 1, 0 as i64);
                        action_prime[(action_prime_count, 0)] = 1;
                        let mut marking_double_prime =
                            DMatrix::from_element(marking_row as usize, 1, 0 as i64);
                        // let tmp_prime = matrix.incidence.clone().ad_mul(&action_prime);
                        let tmp_prime = matrix.incidence.clone() * &action_prime;
                        let mut is_valid_action_prime = true;

                        // Calculate the new marking and whether the action is valid
                        for k in 0..marking_row {
                            marking_double_prime[(k, 0)] =
                                tmp_prime[(k, 0)] + marking_prime[(k, 0)];
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
                            }
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
                    let reward = self.reward_value(
                        marking.clone(),
                        marking_prime.clone(),
                        goal_state.clone(),
                    );
                    match q_matrix.get(&marking) {
                        Some(hshmap) => {
                            match hshmap.get(&(action_count as i64)) {
                                Some(value) => {
                                    new_value = (1.0 - alpha) * value.clone()
                                        + alpha * (reward + gamma * max_action);
                                    let mut tmp_hash_map = hshmap.clone();
                                    tmp_hash_map.insert(action_count as i64, new_value.clone());
                                    q_matrix.insert(marking.clone(), tmp_hash_map);
                                }
                                None => (),
                            };
                        }
                        None => (),
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
                    None => (),
                };
            }
        }

        return q_matrix;
    }
}

#[test]
fn add_transition_with_edge_conditions() {
    let mut net = PetriNet::new("Test".into());
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    let p1 = Place::new("P1".into(), TokenSet::Finite, vec![Data::Task(id2)]);
    let p2 = Place::new(
        "P2".into(),
        TokenSet::Finite,
        vec![Data::Task(id2), Data::Agent(id1)],
    );
    let p3 = Place::new("P3".into(), TokenSet::Finite, vec![Data::Task(id3)]);
    net.places.insert(p1.id, p1.clone());
    net.places.insert(p2.id, p2.clone());
    net.places.insert(p3.id, p3.clone());
    net.add_transition_with_edge_conditions(
        "Added Transition".into(),
        vec![Data::AgentAgnostic],
        |place: &Place| {
            !place.has_data(&vec![Query::Data(Data::Agent(id1))])
                && place.has_data(&vec![Query::Data(Data::Task(id2))])
        },
        |place: &Place| {
            !place.has_data(&vec![Query::Data(Data::Agent(id1))])
                && place.has_data(&vec![Query::Data(Data::Task(id3))])
        },
        1,
        2,
    );
    println!("{:#?}", net);
    assert_eq!(net.transitions.len(), 1);
    net.transitions.iter().for_each(|(_id, transition)| {
        assert_eq!(transition.name, "Added Transition");
        assert_eq!(transition.input.len(), 1);
        assert_eq!(transition.output.len(), 1);
        assert_eq!(
            transition.input.get(&p1.id).unwrap_or(&(0 as usize)),
            &(1 as usize)
        );
        assert_eq!(
            transition.output.get(&p3.id).unwrap_or(&(0 as usize)),
            &(2 as usize)
        );
        assert_eq!(transition.meta_data, vec![Data::AgentAgnostic]);
    });
}

#[test]
fn split_place() {
    let mut net = PetriNet::new("Test".into());
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    let id4 = Uuid::new_v4();
    let id5 = Uuid::new_v4();
    let p1 = Place::new("P1".into(), TokenSet::Finite, vec![Data::Task(id1)]);
    let p2 = Place::new("P2".into(), TokenSet::Finite, vec![Data::Task(id2)]);
    let p3 = Place::new("P3".into(), TokenSet::Finite, vec![Data::Task(id3)]);
    let t1 = Transition {
        id: Uuid::new_v4(),
        name: "T1".into(),
        input: HashMap::from([(p1.id, 1)]),
        output: HashMap::from([(p2.id, 1)]),
        meta_data: vec![],
    };
    let t2 = Transition {
        id: Uuid::new_v4(),
        name: "T2".into(),
        input: HashMap::from([(p2.id, 1)]),
        output: HashMap::from([(p3.id, 1)]),
        meta_data: vec![],
    };
    let p2_id = p2.id;
    net.places.insert(p1.id, p1);
    net.places.insert(p2_id, p2);
    net.places.insert(p3.id, p3);
    net.transitions.insert(t1.id, t1);
    net.transitions.insert(t2.id, t2);
    net.split_place(
        &p2_id,
        vec![
            vec![Data::Agent(id4), Data::AgentSituated(id4)],
            vec![Data::Agent(id5)],
        ],
        |_transition,_split_data| true
    );
    println!("{:#?}", net);
    assert_eq!(net.places.len(), 4);
    assert_eq!(net.transitions.len(), 4);
    assert_eq!(
        net.query_transitions(&vec![
            Query::Data(Data::Agent(id4)),
            Query::Data(Data::AgentSituated(id4))
        ])
        .first()
        .unwrap()
        .input
        .len(),
        1
    );
    assert_eq!(
        net.query_transitions(&vec![
            Query::Data(Data::Agent(id4)),
            Query::Data(Data::AgentSituated(id4))
        ])
        .first()
        .unwrap()
        .output
        .len(),
        1
    );
}

#[test]
fn filtered_split_place() {
    let mut net = PetriNet::new("Test".into());
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    let id4 = Uuid::new_v4();
    let initial = Place::new("Initial".into(), TokenSet::Finite, vec![]);
    let center = Place::new("center".into(), TokenSet::Finite, vec![Data::Target(id1)]);
    let p1 = Place::new("P1".into(), TokenSet::Finite, vec![Data::Hand(id2)]);
    let p2 = Place::new("P2".into(), TokenSet::Finite, vec![Data::Hand(id3)]);
    let p3 = Place::new("P3".into(), TokenSet::Finite, vec![Data::Hand(id4)]);

    let center_id = center.id;

    let t0 = Transition {
        id: Uuid::new_v4(),
        name: "T0".into(),
        input: HashMap::from([(initial.id, 1)]),
        output: HashMap::from([(center.id, 1)]),
        meta_data: vec![],
    };

    let t1 = Transition {
        id: Uuid::new_v4(),
        name: "T1".into(),
        input: HashMap::from([(center.id, 1)]),
        output: HashMap::from([(p1.id, 1)]),
        meta_data: vec![Data::Hand(id2)],
    };

    let t2 = Transition {
        id: Uuid::new_v4(),
        name: "T2".into(),
        input: HashMap::from([(center.id, 1)]),
        output: HashMap::from([(p2.id, 1)]),
        meta_data: vec![Data::Hand(id3)],
    };

    let t3 = Transition {
        id: Uuid::new_v4(),
        name: "T3".into(),
        input: HashMap::from([(center.id, 1)]),
        output: HashMap::from([(p3.id, 1)]),
        meta_data: vec![Data::Hand(id4)],
    };

    let t4 = Transition {
        id: Uuid::new_v4(),
        name: "T4".into(),
        input: HashMap::from([(p1.id, 1)]),
        output: HashMap::from([(center.id, 1)]),
        meta_data: vec![Data::Hand(id2)],
    };

    let t5 = Transition {
        id: Uuid::new_v4(),
        name: "T5".into(),
        input: HashMap::from([(p2.id, 1)]),
        output: HashMap::from([(center.id, 1)]),
        meta_data: vec![Data::Hand(id3)],
    };

    let t6 = Transition {
        id: Uuid::new_v4(),
        name: "T6".into(),
        input: HashMap::from([(p3.id, 1)]),
        output: HashMap::from([(center.id, 1)]),
        meta_data: vec![Data::Hand(id4)],
    };

    net.places.insert(initial.id, initial);
    net.places.insert(center.id, center);
    net.places.insert(p1.id, p1);
    net.places.insert(p2.id, p2);
    net.places.insert(p3.id, p3);
    net.transitions.insert(t0.id, t0);
    net.transitions.insert(t1.id, t1);
    net.transitions.insert(t2.id, t2);
    net.transitions.insert(t3.id, t3);
    net.transitions.insert(t4.id, t4);
    net.transitions.insert(t5.id, t5);
    net.transitions.insert(t6.id, t6);

    net.name_lookup.insert(id1, "Target".into());
    net.name_lookup.insert(id2, "Hand1".into());
    net.name_lookup.insert(id3, "Hand2".into());
    net.name_lookup.insert(id4, "Hand3".into());

    // println!("{:#?}", net);

    net.split_place(
        &center_id,
        vec![
            vec![Data::Standing(id2)],
            vec![Data::Standing(id3)],
        ],
        |transition, split_data| {
            if transition.meta_data.len() == 0 {
                return true
            }
            let poi = split_data.iter().find(|data| data.tag() == DataTag::Standing).unwrap();
            let hand = transition.meta_data.iter().find(|data| data.tag() == DataTag::Hand).unwrap();
            poi.id() == hand.id()
        });

    // println!("{:#?}", net);
    
    assert_eq!(net.places.len(),6);
    assert_eq!(net.transitions.len(),6);
    assert_eq!(net.query_places(&vec![Query::Data(Data::Standing(id2))]).len(),1);
    assert_eq!(net.query_places(&vec![Query::Data(Data::Hand(id2))]).len(),1);
    assert_eq!(net.query_places(&vec![Query::Data(Data::Standing(id3))]).len(),1);
    assert_eq!(net.query_places(&vec![Query::Data(Data::Hand(id3))]).len(),1);
    assert_eq!(net.query_transitions(&vec![Query::Data(Data::Hand(id2))]).len(),2);
    assert_eq!(net.query_transitions(&vec![Query::Data(Data::Hand(id3))]).len(),2);
    assert_eq!(net.query_transitions(&vec![Query::Data(Data::Standing(id2))]).len(),3);
    assert_eq!(net.query_transitions(&vec![Query::Data(Data::Standing(id3))]).len(),3);
    assert_eq!(net.query_transitions(&vec![Query::Data(Data::Hand(id4))]).len(),0);

}