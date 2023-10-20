use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{description::primitive::Primitive, util::split_primitives};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type",rename_all = "camelCase")]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub primitives: Vec<Uuid>,
    pub dependencies: Vec<(Uuid, usize)>,
    pub output: Vec<(Uuid, usize)>,
    pub pois: Vec<Uuid>,
}

impl Task {

    pub fn new(
        name: String, 
        primitives: Vec<Uuid>, 
        dependencies: Vec<(Uuid, usize)>, 
        output: Vec<(Uuid, usize)>, 
        pois: Vec<Uuid>
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            primitives,
            dependencies,
            output,
            pois,
        }
    }

    pub fn new_empty(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            primitives: Vec::new(),
            dependencies: Vec::new(),
            output: Vec::new(),
            pois: Vec::new(),
        }
    }
    
    pub fn set_name(&mut self, name: &String) {
        self.name = name.clone();
    }

    pub fn add_primitive(&mut self, primitive: Uuid) {
        self.primitives.push(primitive);
    }

    pub fn add_dependency(&mut self, target: &Uuid, count: usize) {
        let found_dependencies: Option<(usize, &(Uuid, usize))> = self
            .dependencies
            .iter()
            .enumerate()
            .find(|(_idx, (target_candidate, _count))| target_candidate == target);
        match found_dependencies {
            Some((idx, _)) => {
                self.dependencies[idx].1 += count;
            }
            None => {
                self.dependencies.push((*target, count));
            }
        }
    }

    pub fn add_output(&mut self, target: &Uuid, count: usize) {
        let found_output: Option<(usize, &(Uuid, usize))> = self
            .output
            .iter()
            .enumerate()
            .find(|(_idx, (target_candidate, _count))| target_candidate == target);
        match found_output {
            Some((idx, _)) => {
                self.output[idx].1 += count;
            }
            None => {
                self.output.push((*target, count));
            }
        }
    }

    pub fn add_reusable(&mut self, target: &Uuid, count: usize) {
        self.add_dependency(target, count);
        self.add_output(target, count);
    }

    pub fn add_point_of_interest(&mut self, poi: &Uuid) {
        self.pois.push(*poi);
    }

    pub fn output_target_count(&self, id: &Uuid) -> usize {
        self
                .output
                .iter()
                .filter_map(|(target, count)| if target == id { Some(count) } else { None })
                .sum()
    }

    pub fn get_split(&self, splits: usize, primitives: &HashMap<Uuid,Primitive>) -> Vec<Vec<Uuid>> {
        // let mut split: Vec<Vec<Uuid>> = Vec::new();
        split_primitives(
            &self.primitives.iter().map(|p| primitives.get(p).unwrap()).collect(), splits
        )
    }
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "default".to_string(),
            primitives: Vec::new(),
            dependencies: Vec::new(),
            output: Vec::new(),
            pois: Vec::new(),
        }
    }
}