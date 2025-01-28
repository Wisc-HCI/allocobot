use uuid::Uuid;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::description::{poi::PointOfInterest, agent::Agent, target::Target, rating::Rating};
use enum_tag::EnumTag;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, EnumTag)]
#[serde(tag = "type",rename_all = "camelCase")]
pub enum Primitive {
    // -- Cognitive Primitives --

    // A process by which a target is selected
    Selection {
        id: Uuid,
        target: Uuid,
        // Features
        skill: Rating
    },
    // A process by which a target is observed and asessed according to certain properties
    Inspect {
        id: Uuid,
        target: Uuid,
        // Features
        skill: Rating
    },

    // -- Physical Primitives --

    // A process by which a target is held
    Hold {
        id: Uuid,
        target: Uuid
    },
    // A process by which a hand is repositioned on/about a target
    Position {
        id: Uuid,
        target: Uuid,
        degrees: f64,
        displacement: f64
    },
    // A process by which a tool is used
    Use {
        id: Uuid,
        target: Uuid
    },

    // A process by which force is applied
    Force {
        id: Uuid,
        target: Uuid,
        magnitude: f64
    },

    // Pseudo-Primitives added by Algorithm

    // An agent moves from one POI to another
    Travel {
        id: Uuid,
        from_standing: Uuid,
        to_standing: Uuid,
        from_hand: Uuid,
        to_hand: Uuid
    },

    // An agent moves an object from one Standing/Hand POI to another
    Carry {
        id: Uuid,
        target: Uuid,
        from_standing: Uuid,
        to_standing: Uuid,
        from_hand: Uuid,
        to_hand: Uuid
    },

    Reach {
        id: Uuid,
        standing: Uuid,
        from_hand: Uuid,
        to_hand: Uuid
    },

    // An agent moves an object from one Hand POI to another
    Move {
        id: Uuid,
        target: Uuid,
        standing: Uuid,
        from_hand: Uuid,
        to_hand: Uuid
    }
    
}

impl Primitive {
    pub fn id(&self) -> Uuid {
        match self {
            Primitive::Selection { id, .. } => *id,
            Primitive::Inspect { id, .. } => *id,
            Primitive::Hold { id, .. } => *id,
            Primitive::Position { id, .. } => *id,
            Primitive::Use { id, .. } => *id,
            Primitive::Force { id, .. } => *id,
            Primitive::Travel { id, .. } => *id,
            Primitive::Reach { id, .. } => *id,
            Primitive::Carry { id, .. } => *id,
            Primitive::Move { id, .. } => *id,
        }
    }

    pub fn new_selection(target: Uuid, skill:Rating) -> Self {
        Primitive::Selection {
            id: Uuid::new_v4(),
            target,
            skill
        }
    }

    pub fn new_inspect(target: Uuid, skill:Rating) -> Self {
        Primitive::Inspect {
            id: Uuid::new_v4(),
            target,
            skill
        }
    }

    pub fn new_hold(target: Uuid) -> Self {
        Primitive::Hold {
            id: Uuid::new_v4(),
            target
        }
    }

    pub fn new_position(target: Uuid, degrees: f64, displacement: f64) -> Self {
        Primitive::Position {
            id: Uuid::new_v4(),
            target,
            degrees,
            displacement
        }
    }

    pub fn new_use(target: Uuid) -> Self {
        Primitive::Use {
            id: Uuid::new_v4(),
            target
        }
    }

    pub fn new_force(target: Uuid, magnitude:f64) -> Self {
        Primitive::Force {
            id: Uuid::new_v4(),
            target,
            magnitude
        }
    }

    pub fn target(&self) -> Option<Uuid> {
        match self {
            Primitive::Selection { target, .. } => Some(*target),
            Primitive::Inspect { target, .. } => Some(*target),
            Primitive::Hold { target, .. } => Some(*target),
            Primitive::Position { target, .. } => Some(*target),
            Primitive::Use { target, .. } => Some(*target),
            Primitive::Force { target, .. } => Some(*target),
            Primitive::Carry { target, .. } => Some(*target),
            Primitive::Move { target, .. } => Some(*target),
            Primitive::Travel { .. } => None,
            Primitive::Reach { .. } => None
        }
    }

    pub fn estimate_cost(&self, _target_lookup: HashMap<Uuid,Target>,_hand_poi: PointOfInterest,_agent:Agent) -> f64 {
        0.0
    }

    /// Returns the affiliation of this primitive with another primitive
    /// 
    /// For very low affiliation, produce a weight of 1
    /// For low affiliation, produce a weight of 2
    /// For medium affiliation, produce a weight of 3
    /// For high affiliation, produce a weight of 4
    /// For very high affiliation, produce a weight of 5
    pub fn affiliation(&self,other:&Self) -> usize {
        match (self,other) {
            // Self and other are the same
            (Primitive::Selection { target: target1, .. },Primitive::Selection { target: target2, .. }) => if target1 == target2 { 5 } else { 1 },
            (Primitive::Inspect { target: target1, .. },Primitive::Inspect { target: target2, .. }) => if target1 == target2 { 5 } else { 1 },
            (Primitive::Hold { target: target1, .. },Primitive::Hold { target: target2, .. }) =>  if target1 == target2 { 5 } else { 1 },
            (Primitive::Position { target: target1, .. },Primitive::Position { target: target2, .. }) =>  if target1 == target2 { 5 } else { 1 },
            (Primitive::Use { target: target1, .. },Primitive::Use { target: target2, .. }) =>  if target1 == target2 { 5 } else { 1 },
            (Primitive::Force { target: target1, .. },Primitive::Force { target: target2, .. }) =>  if target1 == target2 { 5 } else { 1 },
            
            // Cross-type Comparisons

            // Selection and Inspect
            (Primitive::Selection { target: target1, ..},Primitive::Inspect { target: target2, .. }) =>  if target1 == target2 { 4 } else { 1 },
            (Primitive::Inspect { target: target1, ..},Primitive::Selection { target: target2, .. }) =>  if target1 == target2 { 4 } else { 1 },

            // Selection and Hold
            (Primitive::Selection { target: target1, ..},Primitive::Hold { target: target2, .. }) =>  if target1 == target2 { 2 } else { 1 },
            (Primitive::Hold { target: target1, ..},Primitive::Selection { target: target2, .. }) =>  if target1 == target2 { 2 } else { 1 },

            // Selection and Position
            (Primitive::Selection { target: target1, ..},Primitive::Position { target: target2, .. }) =>  if target1 == target2 { 2 } else { 1 },
            (Primitive::Position { target: target1, ..},Primitive::Selection { target: target2, .. }) =>  if target1 == target2 { 2 } else { 1 },

            // Selection and Use
            (Primitive::Selection { target: target1, ..},Primitive::Use { target: target2, .. }) =>  if target1 == target2 { 2 } else { 1 },
            (Primitive::Use { target: target1, ..},Primitive::Selection { target: target2, .. }) =>  if target1 == target2 { 2 } else { 1 },

            // Selection and Force
            (Primitive::Selection { target: target1, ..},Primitive::Force { target: target2, .. }) =>  if target1 == target2 { 2 } else { 1 },
            (Primitive::Force { target: target1, ..},Primitive::Selection { target: target2, .. }) =>  if target1 == target2 { 2 } else { 1 },

            // Inspect and Hold
            (Primitive::Inspect { target: target1, ..},Primitive::Hold { target: target2, .. }) =>  if target1 == target2 { 4 } else { 1 },
            (Primitive::Hold { target: target1, ..},Primitive::Inspect { target: target2, .. }) =>  if target1 == target2 { 4 } else { 1 },

            // Inspect and Position
            (Primitive::Inspect { target: target1, ..},Primitive::Position { target: target2, .. }) =>  if target1 == target2 { 4 } else { 1 },
            (Primitive::Position { target: target1, ..},Primitive::Inspect { target: target2, .. }) =>  if target1 == target2 { 4 } else { 1 },

            // Inspect and Use
            (Primitive::Inspect { target: target1, ..},Primitive::Use { target: target2, .. }) =>  if target1 == target2 { 3 } else { 1 },
            (Primitive::Use { target: target1, ..},Primitive::Inspect { target: target2, .. }) =>  if target1 == target2 { 3 } else { 1 },

            // Inspect and Force
            (Primitive::Inspect { target: target1, ..},Primitive::Force { target: target2, .. }) =>  if target1 == target2 { 2 } else { 1 },
            (Primitive::Force { target: target1, ..},Primitive::Inspect { target: target2, .. }) =>  if target1 == target2 { 2 } else { 1 },

            // Hold and Use
            (Primitive::Hold { target: target1, ..},Primitive::Use { target: target2, .. }) =>  if target1 == target2 { 4 } else { 1 },
            (Primitive::Use { target: target1, ..},Primitive::Hold { target: target2, .. }) =>  if target1 == target2 { 4 } else { 1 },

            // Hold and Force
            (Primitive::Hold { target: target1, ..},Primitive::Force { target: target2, .. }) =>  if target1 == target2 { 4 } else { 1 },
            (Primitive::Force { target: target1, ..},Primitive::Hold { target: target2, .. }) =>  if target1 == target2 { 4 } else { 1 },

            // Hold and Position
            (Primitive::Hold { target: target1, ..},Primitive::Position { target: target2, .. }) =>  if target1 == target2 { 5 } else { 1 },
            (Primitive::Position { target: target1, ..},Primitive::Hold { target: target2, .. }) =>  if target1 == target2 { 5 } else { 1 },

            // Position and Use
            (Primitive::Position { target: target1, ..},Primitive::Use { target: target2, .. }) =>  if target1 == target2 { 3 } else { 1 },
            (Primitive::Use { target: target1, ..},Primitive::Position { target: target2, .. }) =>  if target1 == target2 { 3 } else { 1 },

            // Position and Force
            (Primitive::Position { target: target1, ..},Primitive::Force { target: target2, .. }) =>  if target1 == target2 { 2 } else { 1 },
            (Primitive::Force { target: target1, ..},Primitive::Position { target: target2, .. }) =>  if target1 == target2 { 2 } else { 1 },
            
            // Use and Force
            (Primitive::Use { target: target1, ..},Primitive::Force { target: target2, .. }) =>  if target1 == target2 { 2 } else { 1 },
            (Primitive::Force { target: target1, ..},Primitive::Use { target: target2, .. }) =>  if target1 == target2 { 2 } else { 1 },

            (_, _) => 1
        }
    }
}