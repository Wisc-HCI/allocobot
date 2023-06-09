use uuid::Uuid;
use crate::description::target::Target;
use crate::description::task::Task;

#[derive(Clone, Debug, PartialEq)]
pub struct Dependency<'a> {
    pub id: Uuid,
    pub target: &'a Target,
    pub task: &'a Task<'a>,
    pub count: usize
}

impl <'a> Dependency<'a> {
    pub fn new(task: &'a Task, target: &'a Target) -> Self {
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
