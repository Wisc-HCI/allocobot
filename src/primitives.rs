pub enum Primitive {
    // A process by which a target is selected
    Selection {},
    // A process by which a target is grabbed
    Grab {},
    // A process by which a target is released
    Release {},
    // A process by which a target is held
    Hold {},
    // A process by which the agent moves around the space
    Travel {},
    // A process by which the agent extends/retracts its arm(s)
    Reach {},
    // A process by which the agent attaches one part to another
    Fasten {},
    // A process by which the agent moves around the space
    Press {},
    Insertion {},
    Inspection {},
    ToolSwap {},
}