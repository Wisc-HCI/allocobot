use crate::description::target::Target;
use crate::description::poi::PointOfInterest;

#[derive(Clone,Debug, PartialEq)]
pub enum Primitive<'a> {
    // A process by which a target is selected
    Selection {
        target: &'a Target,
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
        target: &'a Target,
        // Features
        structure: f64,
        variability: f64,
        displacement: f64,
        manipulation: f64,
        alignment: f64
    },
    // A process by which a target is released
    Release {
        target: &'a Target,
        // Features
        structure: f64,
        variability: f64,
        manipulation: f64,
        alignment: f64
    },
    // A process by which a target is held
    Hold {
        target: &'a Target,
        // Features
        manipulation: f64,
        alignment: f64
    },
    // A process by which the agent raises its arm, torso, or body part
    Travel {
        poi: &'a PointOfInterest,
        // Features
        structure: f64,
        variability: f64,
        accessibility: f64,
        displacement: f64,
        alignment: f64
    },
    // A process by which the agent extends/retracts its arm(s)
    Reach {
        poi: &'a PointOfInterest,
        // Features
        structure: f64,
        variability: f64,
        accessibility: f64,
        alignment: f64
    },
    // A process by which the agent attaches one part to another
    Fasten {
        base_target: &'a Target,
        attach_target: &'a Target,
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
        target: &'a Target,
        // Features
        structure: f64,
        variability: f64,
        accessibility: f64,
        alignment: f64,
        forces: f64
    },
    // A process by which the agent inserts a target object into a target recepticle/base
    Insert {
        base_target: &'a Target,
        insert_target: &'a Target,
        // Features
        structure: f64,
        variability: f64,
        accessibility: f64,
        alignment: f64,
        forces: f64
    },
    // A process by which the agent separates a target object from a target recepticle/base
    Separate {
        base_target: &'a Target,
        separate_target: &'a Target,
        // Features
        structure: f64,
        variability: f64,
        accessibility: f64,
        alignment: f64,
        forces: f64
    },
    // A process by which a target is observed and asessed according to certain properties
    Inspect {
        target: &'a Target,
        // Features
        structure: f64,
        variability: f64,
        displacement: f64
    },
    // A process by which an agent swaps one tool for another
    ToolSwap {
        target: &'a Target,
        // Features
        manipulation: f64,
        alignment: f64
    },
    // A process by which an agent swaps one tool for another
    ToolStart {
        target: &'a Target,
        // Features
        manipulation: f64,
        alignment: f64
    },
    ToolStop {
        target: &'a Target,
        // Features
        manipulation: f64,
        alignment: f64
    }
}