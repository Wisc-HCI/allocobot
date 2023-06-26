use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::mem::discriminant;
use enum_tag::EnumTag;

#[derive(Clone, Debug, PartialEq, EnumTag, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum Data {
    AgentTaskLockPlace(Uuid),
    TaskPlace(Uuid),
    TaskTransition(Uuid),
    TaskAllocationTransition(Uuid),
    AgentAllocationTransition(Uuid),
    AllocatedTaskPlace(Uuid),
    UnnallocatedTaskPlace(Uuid),
    TargetPlace(Uuid),
    AgentIndeterminitePlace(Uuid),
    AgentAddTransition(Uuid),
    AgentDiscardTransition(Uuid),
    AgentInitialPlace(Uuid),
    AgentDiscardPlace(Uuid),
    AgentTransition(Uuid),
    NonAgentTransition
}

impl Data {
    pub fn fuzzy_eq(&self, other: &Data) -> bool {
        discriminant(self) == discriminant(other)
    }

    pub fn id(&self) -> Option<Uuid> {
        match self {
            Data::AgentTaskLockPlace(id) => Some(*id),
            Data::TaskPlace(id) => Some(*id),
            Data::TaskTransition(id) => Some(*id),
            Data::TaskAllocationTransition(id) => Some(*id),
            Data::AgentAllocationTransition(id) => Some(*id),
            Data::AllocatedTaskPlace(id) => Some(*id),
            Data::UnnallocatedTaskPlace(id) => Some(*id),
            Data::TargetPlace(id) => Some(*id),
            Data::AgentIndeterminitePlace(id) => Some(*id),
            Data::AgentInitialPlace(id) => Some(*id),
            Data::AgentDiscardPlace(id) => Some(*id),
            Data::AgentAddTransition(id) => Some(*id),
            Data::AgentDiscardTransition(id) => Some(*id),
            Data::AgentTransition(id) => Some(*id),
            Data::NonAgentTransition => None
        }
    }
}

pub fn data_subset(data: &Vec<Data>, subset: &Vec<Data>, fuzzy: bool) -> bool {
    match fuzzy {
        false => subset.iter().all(|s| data.contains(s)),
        true => subset.iter().all(|s| data.iter().any(|d| d.fuzzy_eq(s)))
    }
}

pub type DataTag = <Data as EnumTag>::Tag;