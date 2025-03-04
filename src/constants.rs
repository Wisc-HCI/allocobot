// Split size is the total number of agents that can be allocated to a single task at a time.
pub const SPLIT_SIZE: usize = 2;

// Conversion from TMUs to Seconds
pub const TMU_PER_SECOND: f64 = 0.036;

// Distance to Paces
pub const DISTANCE_PER_PACE: f64 = 1.19;

// Number of seconds in an hour
pub const SEC_PER_HOUR: f64 = 3600.0;

// distances for categorizing work type
pub const MAX_HAND_WORK_DISTANCE: f64 = 0.05;
pub const MAX_ARM_WORK_DISTANCE: f64 = 0.45;
pub const MAX_SHOULDER_WORK_DISTANCE: f64 = 0.91;
pub const MAX_FULLBODY_WORK_DISTANCE: f64 = 2.0;