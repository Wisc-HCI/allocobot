use crate::description::dependency::Dependency;
// use crate::description::poi::PointOfInterest;
// use crate::description::primitive::Primitive;
// use crate::description::target::Target;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub enum Task {
    Process(Process),
    Spawn(Spawn),
    Complete(Complete),
}

impl Task {
    pub fn new_process() -> Self {
        Self::Process(Process::new())
    }

    pub fn new_spawn() -> Self {
        Self::Spawn(Spawn::new())
    }

    pub fn new_complete() -> Self {
        Self::Complete(Complete::new())
    }

    pub fn set_name(&mut self, name: &String) {
        match *self {
            Self::Process(ref mut process) => {
                process.name = name.clone();
            }
            Self::Spawn(ref mut spawn) => {
                spawn.name = name.clone();
            }
            Self::Complete(ref mut complete) => {
                complete.name = name.clone();
            }
        }
    }

    pub fn add_primitive(&mut self, primitive: Uuid) {
        match *self {
            Self::Process(ref mut process) => {
                process.primitives.push(primitive);
            }
            _ => {
                println!("Cannot add primitive to Spawn or Complete task");
            }
        }
    }

    pub fn add_dependency(&mut self, task: &Uuid, target: &Uuid) {
        match self {
            Self::Process(ref mut process) => {
                let mut found: bool = false;
                process
                    .dependencies
                    .iter_mut()
                    .for_each(|dependency: &mut Dependency| {
                        if dependency.task == *task && dependency.target == *target {
                            dependency.increment();
                            found = true;
                        }
                    });
                if !found {
                    process.dependencies.push(Dependency::new(*task, *target));
                }
            }
            Self::Complete(ref mut complete) => {
                let mut found: bool = false;
                complete
                    .dependencies
                    .iter_mut()
                    .for_each(|dependency: &mut Dependency| {
                        if dependency.task == *task && dependency.target == *target {
                            dependency.increment();
                            found = true;
                        }
                    });
                if !found {
                    complete.dependencies.push(Dependency::new(*task, *target));
                }
            }
            _ => {
                println!("Cannot add dependency to Spawn task");
            }
        }
    }

    pub fn add_output(&mut self, target: &Uuid, count: usize) {
        match *self {
            Self::Process(ref mut process) => {
                let found_output: Option<(usize, &(Uuid, usize))> = process
                    .output
                    .iter()
                    .enumerate()
                    .find(|(_idx, (target_candidate, _count))| target_candidate == target);
                match found_output {
                    Some((idx, _)) => {
                        process.output[idx].1 += count;
                    }
                    None => {
                        process.output.push((*target, count));
                    }
                }
            }
            Self::Spawn(ref mut spawn) => {
                match spawn.output {
                    Some((spawn_target, spawn_count)) => {
                        if spawn_target == *target {
                            spawn.output = Some((spawn_target, spawn_count + count));
                        } else {
                            spawn.output = Some((*target, count));
                        }
                    }
                    None => {
                        spawn.output = Some((*target, 1));
                    }
                }
            }
            _ => {
                println!("Cannot add output to Complete task");
            }
        }
    }

    pub fn add_poi(&mut self, poi: &Uuid) {
        match *self {
            Self::Process(ref mut process) => {
                process.pois.push(*poi);
            }
            Self::Spawn(ref mut spawn) => {
                spawn.pois.push(*poi);
            }
            Self::Complete(ref mut complete) => {
                complete.pois.push(*poi);
            }
        }
    }

    pub fn id(&self) -> Uuid {
        match self {
            Self::Process(process) => process.id,
            Self::Spawn(spawn) => spawn.id,
            Self::Complete(complete) => complete.id,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Process(process) => process.name.clone(),
            Self::Spawn(spawn) => spawn.name.clone(),
            Self::Complete(complete) => complete.name.clone(),
        }
    }

    pub fn pois(&self) -> Vec<Uuid> {
        match self {
            Self::Process(process) => process.pois.clone(),
            Self::Spawn(spawn) => spawn.pois.clone(),
            Self::Complete(complete) => complete.pois.clone(),
        }
    }

    pub fn primitives(&self) -> Vec<Uuid> {
        match self {
            Self::Process(process) => process.primitives.clone(),
            _ => {
                vec![]
            }
        }
    }

    pub fn dependencies(&self) -> Vec<Dependency> {
        match self {
            Self::Process(process) => process.dependencies.clone(),
            Self::Complete(complete) => complete.dependencies.clone(),
            _ => {
                vec![]
            }
        }
    }

    pub fn output(&self) -> Vec<(Uuid, usize)> {
        match self {
            Self::Process(process) => process.output.iter().map(|target| *target).collect(),
            Self::Spawn(spawn) => spawn.output.map_or(vec![], |target| vec![target]),
            _ => {
                vec![]
            }
        }
    }

    pub fn output_target_count(&self, id: &Uuid) -> usize {
        match self {
            Self::Process(process) => process
                .output
                .iter()
                .filter_map(|(target, count)| if target == id { Some(count) } else { None })
                .sum(),
            Self::Spawn(spawn) => spawn.output.map_or(
                0,
                |(target, count)| if target == *id { count } else { 0 },
            ),
            _ => 0,
        }
    }

    pub fn unique_target_dependencies(&self) -> Vec<Uuid> {
        let mut targets: Vec<Uuid> = vec![];
        match self {
            Self::Process(process) => {
                for dependency in &process.dependencies {
                    if !targets.contains(&dependency.target) {
                        targets.push(dependency.target);
                    }
                }
            }
            Self::Complete(complete) => {
                for dependency in &complete.dependencies {
                    if !targets.contains(&dependency.target) {
                        targets.push(dependency.target);
                    }
                }
            }
            _ => {}
        }
        targets
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Process {
    pub id: Uuid,
    pub name: String,
    pub primitives: Vec<Uuid>,
    pub dependencies: Vec<Dependency>,
    pub output: Vec<(Uuid, usize)>,
    pub pois: Vec<Uuid>,
}

impl Process {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "".into(),
            primitives: vec![],
            dependencies: vec![],
            output: vec![],
            pois: vec![],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Spawn {
    pub id: Uuid,
    pub name: String,
    pub output: Option<(Uuid, usize)>,
    pub pois: Vec<Uuid>,
}

impl Spawn {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "".into(),
            output: None,
            pois: vec![],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Complete {
    pub id: Uuid,
    pub name: String,
    pub dependencies: Vec<Dependency>,
    pub pois: Vec<Uuid>,
}

impl Complete {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "".into(),
            dependencies: vec![],
            pois: vec![],
        }
    }
}
