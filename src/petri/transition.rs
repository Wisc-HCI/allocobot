use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::petri::data::{Data, data_subset};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transition {
    pub id: Uuid,
    pub name: String,
    pub input: HashMap<Uuid,usize>,
    pub output: HashMap<Uuid,usize>,
    pub meta_data: Vec<Data>,
}

impl Transition {
    pub fn new(name: String, meta_data: Vec<Data>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            input: HashMap::new(),
            output: HashMap::new(),
            meta_data
        }
    }

    pub fn with_input(mut self, place_id: &Uuid, tokens: usize) -> Self {
        if self.input.contains_key(place_id) {
            self.input.insert(*place_id, self.input.get(place_id).unwrap() + tokens);
        } else {
            self.input.insert(*place_id, tokens);
        }
        self
    }

    pub fn with_output(mut self, place_id: &Uuid, tokens: usize) -> Self {
        if self.output.contains_key(place_id) {
            self.output.insert(*place_id, self.output.get(place_id).unwrap() + tokens);
        } else {
            self.output.insert(*place_id, tokens);
        }
        self
    }

    pub fn has_data(&self, meta_data: &Vec<Data>, fuzzy: bool) -> bool {
        data_subset(&self.meta_data, meta_data, fuzzy)
    }
}

#[test]
pub fn data_query_mismatched_inner_nonfuzzy() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let transition = Transition::new(
        "test".to_string(),
        vec![
            Data::Task(uuid1),
            Data::Agent(uuid2)
        ]
    );
    assert_eq!(transition.has_data(&vec![Data::Task(uuid2)], false), false);
}

#[test]
pub fn data_query_mismatched_inner_fuzzy() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let transition = Transition::new(
        "test".to_string(),
        vec![
            Data::Task(uuid1),
            Data::Agent(uuid2)
        ]
    );
    assert_eq!(transition.has_data(&vec![Data::Task(uuid2)], true), true);
}

#[test]
pub fn data_query_matched_inner_nonfuzzy() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let transition = Transition::new(
        "test".to_string(),
        vec![
            Data::Task(uuid1),
            Data::Agent(uuid2)
        ]
    );
    assert_eq!(transition.has_data(&vec![Data::Task(uuid1)], false), true);
}