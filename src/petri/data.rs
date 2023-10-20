use enum_tag::EnumTag;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
// use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, EnumTag, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum Data {
    // Contain Agent UUID
    Agent(Uuid),
    AgentPresent(Uuid),
    AgentSituated(Uuid),
    AgentIndeterminite(Uuid),
    AgentDiscard(Uuid),
    AgentTaskLock(Uuid),
    AgentAdd(Uuid),
    AgentJoint,

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

    // Primitive Assignments
    // Encoded as Agent UUID, Primitive UUID
    PrimitiveAssignment(Uuid, Uuid),

    // Contain No UUID
    AgentAgnostic,

    // Cost-Related
    Action(Uuid), // A meta-data that includes tasks or anything physical, likely with a cost
    ErgoWholeBody(Uuid, usize),
    ErgoArm(Uuid, usize),
    ErgoHand(Uuid, usize),
}

impl Data {
    pub fn fuzzy_eq(&self, other: &Data) -> bool {
        self.tag() == other.tag()
    }

    pub fn id(&self) -> Option<Uuid> {
        match self {
            Data::Agent(id) => Some(*id),
            Data::AgentPresent(id) => Some(*id),
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
            // PrimitiveAssignment returns the Agent UUID
            Data::PrimitiveAssignment(id, _) => Some(*id),
            Data::AgentAgnostic => None,
            Data::AgentJoint => None,
            Data::Action(id) => Some(*id),
            Data::ErgoWholeBody(id, _) => Some(*id),
            Data::ErgoArm(id, _) => Some(*id),
            Data::ErgoHand(id, _) => Some(*id),
        }
    }

    pub fn secondary(&self) -> Option<Uuid> {
        match self {
            Data::Agent(_id) => None,
            Data::AgentPresent(_id) => None,
            Data::AgentSituated(_id) => None,
            Data::AgentIndeterminite(_id) => None,
            Data::AgentDiscard(_id) => None,
            Data::AgentTaskLock(_id) => None,
            Data::AgentAdd(_id) => None,
            Data::Task(_id) => None,
            Data::UnnallocatedTask(_id) => None,
            Data::AllocatedTask(_id) => None,
            Data::Target(_id) => None,
            Data::TargetUnplaced(_id) => None,
            Data::TargetSituated(_id) => None,
            Data::Standing(_id) => None,
            Data::Hand(_id) => None,
            Data::FromStandingPOI(_id) => None,
            Data::ToStandingPOI(_id) => None,
            Data::FromHandPOI(_id) => None,
            Data::ToHandPOI(_id) => None,
            // PrimitiveAssignment returns the Primitive UUID
            Data::PrimitiveAssignment(_, id) => Some(*id),
            Data::AgentAgnostic => None,
            Data::AgentJoint => None,
            Data::Action(_id) => None,
            Data::ErgoWholeBody(_, _) => None,
            Data::ErgoArm(_, _) => None,
            Data::ErgoHand(_, _) => None,
        }
    }

    pub fn numeric(&self) -> Option<usize> {
        match self {
            Data::Agent(_id) => None,
            Data::AgentPresent(_id) => None,
            Data::AgentSituated(_id) => None,
            Data::AgentIndeterminite(_id) => None,
            Data::AgentDiscard(_id) => None,
            Data::AgentTaskLock(_id) => None,
            Data::AgentAdd(_id) => None,
            Data::Task(_id) => None,
            Data::UnnallocatedTask(_id) => None,
            Data::AllocatedTask(_id) => None,
            Data::Target(_id) => None,
            Data::TargetUnplaced(_id) => None,
            Data::TargetSituated(_id) => None,
            Data::Standing(_id) => None,
            Data::Hand(_id) => None,
            Data::FromStandingPOI(_id) => None,
            Data::ToStandingPOI(_id) => None,
            Data::FromHandPOI(_id) => None,
            Data::ToHandPOI(_id) => None,
            // PrimitiveAssignment returns the Primitive UUID
            Data::PrimitiveAssignment(_, _) => None,
            Data::AgentAgnostic => None,
            Data::AgentJoint => None,
            Data::Action(_id) => None,
            Data::ErgoWholeBody(_, n) => Some(*n),
            Data::ErgoArm(_, n) => Some(*n),
            Data::ErgoHand(_, n) => Some(*n),
        }
    }
}

pub fn data_query(data: &Vec<Data>, query: &Vec<Query>) -> bool {
    query.iter().all(|q| match q {
        Query::Data(d) => data.contains(d),
        Query::Tag(t) => data.iter().any(|d| d.tag() == *t),
    })
}

pub type DataTag = <Data as EnumTag>::Tag;

pub enum Query {
    Data(Data),
    Tag(DataTag),
}

// #[derive(Clone, Debug, Eq, Hash, Serialize, Deserialize)]
// pub struct Clade {
//     pub id: Uuid,
//     pub name: String,
//     pub children: Vec<Clade>,
// }

// impl Clade {
//     pub fn new(name: String, children: Vec<Clade>) -> Clade {
//         Clade {
//             id: Uuid::new_v4(),
//             name,
//             children
//         }
//     }

//     pub fn add_child(&mut self, child: &Clade) {
//         self.children.push(child.clone());
//     }

//     pub fn descendent(&self, id: &Uuid) -> bool {
//         if self.id == *id {
//             return true;
//         }
//         return self.children.iter().any(|c| c.descendent(id));
//     }

//     pub fn query(&self, name_query: &String) -> Option<Uuid> {
//         if *name_query == self.name {
//             return Some(self.id);
//         }
//         for child in self.children.iter() {
//             match child.query(name_query) {
//                 Some(id) => return Some(id),
//                 None => continue,
//             }
//         }
//         return None;
//     }

//     pub fn get(&self, id: &Uuid) -> Option<Clade> {
//         if self.id == *id {
//             return Some(self.clone());
//         }
//         for child in self.children.iter() {
//             match child.get(id) {
//                 Some(c) => return Some(c),
//                 None => continue,
//             }
//         }
//         return None;
//     }
// }

// impl PartialOrd for Clade {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         match (self.descendent(&other.id), other.descendent(&self.id)) {
//             (true, true) => Some(Ordering::Equal),
//             (true, false) => Some(Ordering::Greater),
//             (false, true) => Some(Ordering::Less),
//             (false, false) => None,
//         }
//     }
// }

// impl PartialEq for Clade {
//     fn eq(&self, other: &Self) -> bool {
//         self.id == other.id
//     }
// }

// #[test]
// pub fn clade_descendents() {
//     let tax1 = Clade::new(
//         "root:1".into(),
//         vec![Clade::new(
//             "child:1".into(),
//             vec![
//                 Clade::new("grandchild1:1".into(), vec![]),
//                 Clade::new("grandchild2:1".into(), vec![]),
//             ],
//         )],
//     );

//     let tax2 = Clade::new(
//         "root:2".into(),
//         vec![Clade::new(
//             "child:2".into(),
//             vec![]
//         )],
//     );

//     println!("tax 1: {:?}",tax1);
//     assert!(tax1.descendent(&tax1.id));
//     assert!(tax1 >= tax1);
//     assert!(tax1 == tax1);
//     assert!(tax1.children[0].descendent(&tax1.children[0].id));
//     assert!(tax1.descendent(&tax1.children[0].id));
//     assert!(!(tax1 < tax1.children[0]));
//     assert!(tax1 > tax1.children[0]);
//     assert!(tax1 >= tax1.children[0]);
//     assert!(tax1.children[0] == tax1.children[0]);
//     let grandchildren = tax1.children[0].children.clone();
//     assert_eq!(grandchildren.len(), 2);
//     assert!(grandchildren[0].descendent(&grandchildren[0].id));
//     assert!(tax1.children[0].descendent(&grandchildren[0].id));
//     assert!(tax1.children[0] > grandchildren[0]);
//     assert!(tax1.children[0] >= grandchildren[1]);
//     assert!(tax1 > grandchildren[0]);
//     assert!(!(tax1 < grandchildren[0]));
//     assert!(grandchildren[0] != grandchildren[1]);
//     assert!(tax1.descendent(&grandchildren[0].id));
//     assert!(tax1.descendent(&grandchildren[1].id));
//     assert_eq!(tax1.children[0].id,tax1.query(&"child:1".into()).unwrap());
//     assert_eq!(tax1.children[0],tax1.get(&tax1.children[0].id).unwrap());

// }

// #[test]
// pub fn data_as_clades() {
//     let charlie_present: Clade = Clade::new("Charlie Present".into(),vec![]);
//     let panda_present: Clade = Clade::new("Panda Present".into(),vec![]);
//     let charlie_situated: Clade = Clade::new("Charlie Situated".into(),vec![]);
//     let panda_situated: Clade = Clade::new("Panda Situated".into(),vec![]);
//     let charlie_discard: Clade = Clade::new("Charlie Discard".into(),vec![]);
//     let panda_discard: Clade = Clade::new("Panda Discard".into(),vec![]);

//     let charlie: Clade = Clade::new("Charlie".into(),vec![charlie_present.clone(),charlie_situated.clone(),charlie_discard.clone()]);
//     let panda: Clade = Clade::new("Panda".into(),vec![panda_present.clone(),panda_situated.clone(),panda_discard.clone()]);
    
//     let agent_present: Clade = Clade::new("Agent Present".into(),vec![charlie_present.clone(),panda_present.clone()]);
//     let agent_situated: Clade = Clade::new("Agent Situated".into(),vec![charlie_situated.clone(),panda_situated.clone()]);
//     let agent_agnostic: Clade = Clade::new("Agent Agnostic".into(),vec![]);
//     let agent: Clade = Clade::new("agent".into(),vec![agent_present.clone(),agent_situated.clone()]);

//     let task: Clade = Clade::new("task".into(),vec![]);

// }
