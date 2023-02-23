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
    Lift {},
    // A process by which the agent raises its arm, torso, or body part
    Travel {},
    // A process by which the agent extends/retracts its arm(s)
    Reach {},
    // A process by which the agent attaches one part to another
    Position {},
    // A process by which an object is turned or rotated
    Fasten {},
    // A process by which the agent applies pressure on a target or surface
    Press {},
    // A process by which the agent inserts a target object into a target recepticle
    Insert {},
    // A process by which a target is observed and asessed according to certain properties
    Inspect {},
    // A process by which an agent swaps one tool for another
    ToolSwap {},
}