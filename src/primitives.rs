#[derive(Clone,Debug)]
pub enum Primitive {
    // A process by which a target is selected
    Selection {
        target: String,
        // Features
        structure: f64,
        variability: f64,
        displacement: f64
    },
    // A process by which a target is grabbed
    Grab {
        target: String,
        // Features
        structure: f64,
        variability: f64,
        manipulation: f64,
        alignment: f64
    },
    // A process by which a target is released
    Release {
        target: String,
        // Features
        structure: f64,
        variability: f64,
        manipulation: f64,
        alignment: f64
    },
    // A process by which a target is held
    Hold {
        target: String,
        // Features
        manipulation: f64,
        alignment: f64,
        distance: f64
    },
    // A process by which the agent moves around the space
    Lift {
        // Features
        manipulation: f64,
        alignment: f64,
        distance: f64
    },
    // A process by which the agent raises its arm, torso, or body part
    Travel {
        // Features
        structure: f64,
        variability: f64,
        accessibility: f64,
        displacement: f64,
        alignment: f64,
        distance: f64,
        mobility: bool
    },
    // A process by which the agent extends/retracts its arm(s)
    Reach {
        // Features
        structure: f64,
        variability: f64,
        accessibility: f64,
        displacement: f64,
        alignment: f64,
        distance: f64,
        mobility: bool
    },
    // A process by which an object is turned or rotated
    Position {
        // Features
        structure: f64,
        variability: f64,
        displacement: f64,
        manipulation: f64,
        alignment: f64
    },
    // A process by which the agent attaches one part to another
    Fasten {
        base_target: String,
        attach_target: String,
        // Features
        structure: f64,
        variability: f64,
        accessibility: f64,
        displacement: f64,
        manipulation: f64,
        alignment: f64,
        forces: f64,
        mobility: bool
    },
    // A process by which the agent applies pressure on a target or surface
    Press {
        target: String,
        // Features
        structure: f64,
        variability: f64,
        accessibility: f64,
        alignment: f64,
        forces: f64,
        mobility: bool
    },
    // A process by which the agent inserts a target object into a target recepticle/base
    Insert {
        base_target: String,
        insert_target: String,
        // Features
        structure: f64,
        variability: f64,
        accessibility: f64,
        alignment: f64,
        forces: f64,
        mobility: bool
    },
    // A process by which a target is observed and asessed according to certain properties
    Inspect {
        target: String,
        // Features
        structure: f64,
        variability: f64,
        displacement: f64,
        mobility: bool
    },
    // A process by which an agent swaps one tool for another
    ToolSwap {
        target: String,
        // Features
        manipulation: f64,
        alignment: f64
    },
}