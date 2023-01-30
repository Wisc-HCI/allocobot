use crate::agents::*;
use crate::primitives::*;

pub enum Feature {
    // How 'structured' or organized the workspace and task is
    Structure {

    },
    // How much variability in the workspace and targets may have from action to action
    Deviation {

    },
    // How difficult it is to reach goals due to space constraints
    Accessibility {

    },
    // The extent to which the worker may need to account for relative motion of a target
    Displacement {

    },
    // Specifics about the size, weight, or graspability of the target
    Target {

    },
    // The extent to which certain motion constraints must be followed while performing an action
    MotionConstraints {

    },
    // The amount of distance that must be travelled
    Distance {

    },
    // The amount and type of forces being applied
    Forces {

    },
    // Whether floor travel is allowed to account for reachability limitations.
    Mobility {

    },
}

pub trait Ratable {
    fn assess_robot(_robot: &Robot, _primitive: &Primitive) -> f64 {
        return 0.0;
    }

    fn assess_worker(_worker: &Worker, _primitive: &Primitive) -> f64 {
        return 0.0;
    }
}