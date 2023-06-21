use uuid::Uuid;
use std::mem::discriminant;

#[derive(Clone, Debug, PartialEq)]
pub enum Data {
    TaskLock(Uuid),
    TaskPlace(Uuid),
    TaskTransition(Uuid),
    TargetPlace(Uuid),
    AgentInitial(Uuid),
    AgentTransition(Uuid),
    NonAgentTranstion
}

impl Data {
    pub fn fuzzy_eq(&self, other: &Data) -> bool {
        discriminant(self) == discriminant(other)
    }

    pub fn is_task_lock(&self) -> bool {
        match self {
            Data::TaskLock(_) => true,
            _ => false
        }
    }

    pub fn is_task_place(&self) -> bool {
        match self {
            Data::TaskPlace(_) => true,
            _ => false
        }
    }

    pub fn is_task_transition(&self) -> bool {
        match self {
            Data::TaskTransition(_) => true,
            _ => false
        }
    }

    pub fn is_target_place(&self) -> bool {
        match self {
            Data::TargetPlace(_) => true,
            _ => false
        }
    }

    pub fn is_agent_initial(&self) -> bool {
        match self {
            Data::AgentInitial(_) => true,
            _ => false
        }
    }

    pub fn is_agent_transition(&self) -> bool {
        match self {
            Data::AgentTransition(_) => true,
            _ => false
        }
    }

    pub fn is_non_agent_transition(&self) -> bool {
        match self {
            Data::NonAgentTranstion => true,
            _ => false
        }
    }

    pub fn id(&self) -> Option<Uuid> {
        match self {
            Data::TaskLock(id) => Some(*id),
            Data::TaskPlace(id) => Some(*id),
            Data::TaskTransition(id) => Some(*id),
            Data::TargetPlace(id) => Some(*id),
            Data::AgentInitial(id) => Some(*id),
            Data::AgentTransition(id) => Some(*id),
            Data::NonAgentTranstion => None
        }
    }
}

pub fn data_subset(data: &Vec<Data>, subset: &Vec<Data>, fuzzy: bool) -> bool {
    match fuzzy {
        false => subset.iter().all(|s| data.contains(s)),
        true => subset.iter().all(|s| data.iter().any(|d| d.fuzzy_eq(s)))
    }
}