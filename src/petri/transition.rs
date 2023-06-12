use std::collections::HashMap;
use uuid::Uuid;
// use crate::petri::place::Place;
// use crate::description::task::Task;

#[derive(Clone, Debug, PartialEq)]
pub struct Transition {
    pub id: Uuid,
    pub name: String,
    pub input: HashMap<Uuid,usize>,
    pub output: HashMap<Uuid,usize>,
    pub source_task: Option<Uuid>,
}

impl Transition {
    pub fn new(name: String, source_task: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            input: HashMap::new(),
            output: HashMap::new(),
            source_task
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
}