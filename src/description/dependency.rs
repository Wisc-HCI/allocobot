use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    pub id: Uuid,
    pub target: Uuid,
    pub task: Uuid,
    pub count: usize
}

impl Dependency {
    pub fn new(task: Uuid, target: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            target,
            task,
            count: 1
        }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }
}
