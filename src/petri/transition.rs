use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::petri::data::{Data, Query, data_query};
#[cfg(test)]
use crate::petri::data::DataTag;
use std::ops::{Add,Sub};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum Signature {
    Static(usize),
    Range(usize, usize),
}

impl Default for Signature {
    fn default() -> Self {
        Self::Static(0)
    }
}

impl Add for Signature {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Self::Static(a), Self::Static(b)) => Self::Static(a + b),
            (Self::Static(a), Self::Range(b, c)) => Self::Range(a + b, a + c),
            (Self::Range(a, b), Self::Static(c)) => Self::Range(a + c, b + c),
            (Self::Range(a, b), Self::Range(c, d)) => Self::Range(a + c, b + d),
        }
    }
}

impl Sub for Signature {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (Self::Static(a), Self::Static(b)) => Self::Static(a - b),
            (Self::Static(a), Self::Range(b, c)) => Self::Range(a - b, a - c),
            (Self::Range(a, b), Self::Static(c)) => Self::Range(a - c, b - c),
            (Self::Range(a, b), Self::Range(c, d)) => Self::Range(a - c, b - d),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Transition {
    pub id: Uuid,
    pub name: String,
    pub input: HashMap<Uuid,Signature>,
    pub output: HashMap<Uuid,Signature>,
    pub meta_data: Vec<Data>,
    pub time: usize,
    pub cost: usize,
}

impl Transition {
    pub fn new(
        name: String, 
        input: HashMap<Uuid,Signature>, 
        output: HashMap<Uuid,Signature>, 
        meta_data: Vec<Data>,
        time: usize,
        cost: usize,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            input,
            output,
            meta_data,
            time,
            cost,
        }
    }

    pub fn add_input(mut self, place_id: &Uuid, tokens: usize) -> Self {
        if self.input.contains_key(place_id) {
            self.input.insert(*place_id, self.input.get(place_id).unwrap().clone() + Signature::Static(tokens));
        } else {
            self.input.insert(*place_id, Signature::Static(tokens));
        }
        self
    }

    pub fn add_output(mut self, place_id: &Uuid, tokens: usize) -> Self {
        if self.output.contains_key(place_id) {
            self.output.insert(*place_id, self.output.get(place_id).unwrap().clone() + Signature::Static(tokens));
        } else {
            self.output.insert(*place_id, Signature::Static(tokens));
        }
        self
    }

    pub fn has_data(&self, query_vec: &Vec<Query>) -> bool {
        data_query(&self.meta_data, query_vec)
    }
}

#[test]
pub fn data_query_mismatched_inner_nonfuzzy() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let transition = Transition::new(
        "test".to_string(),
        HashMap::new(),
        HashMap::new(),
        vec![
            Data::Task(uuid1),
            Data::Agent(uuid2)
        ],
        0,
        0,
    );
    assert_eq!(transition.has_data(&vec![Query::Data(Data::Task(uuid2))]), false);
}

#[test]
pub fn data_query_mismatched_inner_fuzzy() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let transition = Transition::new(
        "test".to_string(),
        HashMap::new(),
        HashMap::new(),
        vec![
            Data::Task(uuid1),
            Data::Agent(uuid2)
        ],
        0,
        0,
    );
    assert_eq!(transition.has_data(&vec![Query::Tag(DataTag::Task)]), true);
}

#[test]
pub fn data_query_matched_inner_nonfuzzy() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let transition = Transition::new(
        "test".to_string(),
        HashMap::new(),
        HashMap::new(),
        vec![
            Data::Task(uuid1),
            Data::Agent(uuid2)
        ],
        0,
        0
    );
    assert_eq!(transition.has_data(&vec![Query::Data(Data::Task(uuid1))]), true);
}