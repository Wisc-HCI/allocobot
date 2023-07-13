use serde::{Serialize, Deserialize};
use uuid::Uuid;
use enum_tag::EnumTag;

#[derive(Clone, Debug, PartialEq, EnumTag, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum Data {
    // Contain Agent UUID
    Agent(Uuid),
        AgentSituated(Uuid),
        AgentIndeterminite(Uuid),
        AgentDiscard(Uuid),
        AgentTaskLock(Uuid),
        AgentAdd(Uuid),
        
    // Contain Task UUID
    Task(Uuid),
        UnnallocatedTask(Uuid),
        AllocatedTask(Uuid),

    // Contain Target UUID
    Target(Uuid),
        TargetUnplaced(Uuid),
        TargetSituated(Uuid),

    // Contain POI UUID
    Standing(Uuid),
    Hand(Uuid),
    FromStandingPOI(Uuid),
    ToStandingPOI(Uuid),
    FromHandPOI(Uuid),
    ToHandPOI(Uuid),
    
    // Contain No UUID
    AgentAgnostic
}

impl Data {
    pub fn fuzzy_eq(&self, other: &Data) -> bool {
        self.tag() == other.tag()
    }

    pub fn id(&self) -> Option<Uuid> {
        match self {
            Data::Agent(id) => Some(*id),
            Data::AgentSituated(id) => Some(*id),
            Data::AgentIndeterminite(id) => Some(*id),
            Data::AgentDiscard(id) => Some(*id),
            Data::AgentTaskLock(id) => Some(*id),
            Data::AgentAdd(id) => Some(*id),
            Data::Task(id) => Some(*id),
            Data::UnnallocatedTask(id) => Some(*id),
            Data::AllocatedTask(id) => Some(*id),
            Data::Target(id) => Some(*id),
            Data::TargetUnplaced(id) => Some(*id),
            Data::TargetSituated(id) => Some(*id),
            Data::Standing(id) => Some(*id),
            Data::Hand(id) => Some(*id),
            Data::FromStandingPOI(id) => Some(*id),
            Data::ToStandingPOI(id) => Some(*id),
            Data::FromHandPOI(id) => Some(*id),
            Data::ToHandPOI(id) => Some(*id),
            Data::AgentAgnostic => None
        }
    }
}

pub fn data_query(data: &Vec<Data>, query: &Vec<Query>) -> bool {
    query.iter().all(|q| {
        match q {
            Query::Data(d) => data.contains(d),
            Query::Tag(t) => data.iter().any(|d| d.tag() == *t)
        }
    })
    // match fuzzy {
    //     false => subset.iter().all(|s| data.contains(s)),
    //     true => subset.iter().all(|s| data.iter().any(|d| d.fuzzy_eq(s)))
    // }
}

pub type DataTag = <Data as EnumTag>::Tag;

pub enum Query {
    Data(Data),
    Tag(DataTag)
}