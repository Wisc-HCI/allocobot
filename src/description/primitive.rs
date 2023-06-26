use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type",rename_all = "camelCase")]
pub enum Primitive {
    // A process by which a target is selected
    Selection {
        id: Uuid,
        target: Uuid,
        // Rules
        /*
        pre(agent) -> !busy(agent)
        during(agent) -> busy(agent)
        post(agent) -> !busy(agent)
        */
        // Features
        structure: f64,
        variability: f64,
        displacement: f64
    },
    // A process by which a target is grabbed
    Grasp {
        id: Uuid,
        target: Uuid,
        // Features
        structure: f64,
        variability: f64,
        displacement: f64,
        manipulation: f64,
        alignment: f64
    },
    // A process by which a target is released
    Release {
        id: Uuid,
        target: Uuid,
        // Features
        structure: f64,
        variability: f64,
        manipulation: f64,
        alignment: f64
    },
    // A process by which a target is held
    Hold {
        id: Uuid,
        target: Uuid,
        // Features
        manipulation: f64,
        alignment: f64
    },
    // A process by which the agent raises its arm, torso, or body part
    Travel {
        id: Uuid,
        poi: Uuid,
        // Features
        structure: f64,
        variability: f64,
        accessibility: f64,
        displacement: f64,
        alignment: f64
    },
    // A process by which the agent extends/retracts its arm(s)
    Reach {
        id: Uuid,
        poi: Uuid,
        // Features
        structure: f64,
        variability: f64,
        accessibility: f64,
        alignment: f64
    },
    // A process by which the agent attaches one part to another
    Fasten {
        id: Uuid,
        base_target: Uuid,
        attach_target: Uuid,
        // Features
        structure: f64,
        variability: f64,
        accessibility: f64,
        displacement: f64,
        manipulation: f64,
        alignment: f64,
        forces: f64
    },
    // A process by which the agent applies pressure on a target or surface
    Press {
        id: Uuid,
        target: Uuid,
        // Features
        structure: f64,
        variability: f64,
        accessibility: f64,
        alignment: f64,
        forces: f64
    },
    // A process by which the agent inserts a target object into a target recepticle/base
    Insert {
        id: Uuid,
        base_target: Uuid,
        insert_target: Uuid,
        // Features
        structure: f64,
        variability: f64,
        accessibility: f64,
        alignment: f64,
        forces: f64
    },
    // A process by which the agent separates a target object from a target recepticle/base
    Separate {
        id: Uuid,
        base_target: Uuid,
        separate_target: Uuid,
        // Features
        structure: f64,
        variability: f64,
        accessibility: f64,
        alignment: f64,
        forces: f64
    },
    // A process by which a target is observed and asessed according to certain properties
    Inspect {
        id: Uuid,
        target: Uuid,
        // Features
        structure: f64,
        variability: f64,
        displacement: f64
    },
    // A process by which an agent swaps one tool for another
    ToolSwap {
        id: Uuid,
        target: Uuid,
        // Features
        manipulation: f64,
        alignment: f64
    },
    // A process by which an agent swaps one tool for another
    ToolStart {
        id: Uuid,
        target: Uuid,
        // Features
        manipulation: f64,
        alignment: f64
    },
    ToolStop {
        id: Uuid,
        target: Uuid,
        // Features
        manipulation: f64,
        alignment: f64
    }
}

impl Primitive {
    pub fn id(&self) -> Uuid {
        match self {
            Primitive::Selection { id, .. } => *id,
            Primitive::Grasp { id, .. } => *id,
            Primitive::Release { id, .. } => *id,
            Primitive::Hold { id, .. } => *id,
            Primitive::Travel { id, .. } => *id,
            Primitive::Reach { id, .. } => *id,
            Primitive::Fasten { id, .. } => *id,
            Primitive::Press { id, .. } => *id,
            Primitive::Insert { id, .. } => *id,
            Primitive::Separate { id, .. } => *id,
            Primitive::Inspect { id, .. } => *id,
            Primitive::ToolSwap { id, .. } => *id,
            Primitive::ToolStart { id, .. } => *id,
            Primitive::ToolStop { id, .. } => *id
        }
    }
}