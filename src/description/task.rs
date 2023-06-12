use crate::description::dependency::Dependency;
use crate::description::poi::PointOfInterest;
use crate::description::primitive::Primitive;
use crate::description::target::Target;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub enum Task<'a> {
    Process(Process<'a>),
    Spawn(Spawn<'a>),
    Complete(Complete<'a>),
}

impl<'a> Task<'a> {
    pub fn new_process() -> Self {
        Self::Process(Process::new())
    }

    pub fn new_spawn() -> Self {
        Self::Spawn(Spawn::new())
    }

    pub fn new_complete() -> Self {
        Self::Complete(Complete::new())
    }

    pub fn with_name(self, name: String) -> Self {
        match self {
            Self::Process(mut process) => {
                process.name = name;
                Self::Process(process)
            }
            Self::Spawn(mut spawn) => {
                spawn.name = name;
                Self::Spawn(spawn)
            }
            Self::Complete(mut complete) => {
                complete.name = name;
                Self::Complete(complete)
            }
        }
    }

    pub fn with_primitive(self, primitive: Primitive<'a>) -> Self {
        match self {
            Self::Process(mut process) => {
                process.primitives.push(primitive);
                Self::Process(process)
            }
            _ => {
                println!("Cannot add primitive to Spawn or Complete task");
                self
            }
        }
    }

    pub fn with_dependency(self, task: &'a Task<'a>, target: &'a Target) -> Self {
        match self {
            Self::Process(mut process) => {
                let mut found: bool = false;
                process
                    .dependencies
                    .iter_mut()
                    .for_each(|dependency: &mut Dependency| {
                        if dependency.task == task && dependency.target == target {
                            dependency.increment();
                            found = true;
                        }
                    });
                if !found {
                    process.dependencies.push(Dependency::new(task, target));
                }
                Self::Process(process)
            }
            Self::Complete(mut complete) => {
                let mut found: bool = false;
                complete
                    .dependencies
                    .iter_mut()
                    .for_each(|dependency: &mut Dependency| {
                        if dependency.task == task && dependency.target == target {
                            dependency.increment();
                            found = true;
                        }
                    });
                if !found {
                    complete.dependencies.push(Dependency::new(task, target));
                }
                Self::Complete(complete)
            }
            _ => {
                println!("Cannot add dependency to Spawn task");
                self
            }
        }
    }

    pub fn with_output(self, target: &'a Target, count: usize) -> Self {
        match self {
            Self::Process(mut process) => {
                let mut found_output: Option<(usize, &(&Target, usize))> = process
                    .output
                    .iter()
                    .enumerate()
                    .find(|(idx, target_pair)| target_pair.0.id == target.id);
                match found_output {
                    Some((idx, _)) => {
                        process.output[idx].1 += count;
                    }
                    None => {
                        process.output.push((target, count));
                    }
                }
                Self::Process(process)
            }
            Self::Spawn(mut spawn) => {
                match spawn.output {
                    Some(o) => {
                        if o.0.id == target.id {
                            spawn.output = Some((o.0, o.1 + 1));
                        } else {
                            spawn.output = Some((target, 1));
                        }
                    }
                    None => {
                        spawn.output = Some((target, 1));
                    }
                }
                spawn.output = Some((target, 1));
                Self::Spawn(spawn)
            }
            _ => {
                println!("Cannot add output to Complete task");
                self
            }
        }
    }

    pub fn with_poi(self, poi: &'a PointOfInterest) -> Self {
        match self {
            Self::Process(mut process) => {
                process.pois.push(poi);
                Self::Process(process)
            }
            Self::Spawn(mut spawn) => {
                spawn.pois.push(poi);
                Self::Spawn(spawn)
            }
            Self::Complete(mut complete) => {
                complete.pois.push(poi);
                Self::Complete(complete)
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

    pub fn pois(&self) -> Vec<&PointOfInterest> {
        match self {
            Self::Process(process) => process.pois.clone(),
            Self::Spawn(spawn) => spawn.pois.clone(),
            Self::Complete(complete) => complete.pois.clone(),
        }
    }

    pub fn primitives(&self) -> Vec<&Primitive> {
        match self {
            Self::Process(process) => process.primitives.iter().collect(),
            _ => {
                vec![]
            }
        }
    }

    pub fn dependencies(&self) -> Vec<&Dependency> {
        match self {
            Self::Process(process) => process.dependencies.iter().collect(),
            Self::Complete(complete) => complete.dependencies.iter().collect(),
            _ => {
                vec![]
            }
        }
    }

    pub fn output(&self) -> Vec<(&Target, usize)> {
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
                .filter_map(|(target, count)| if target.id == *id { Some(count) } else { None })
                .sum(),
            Self::Spawn(spawn) => spawn.output.map_or(
                0,
                |(target, count)| if target.id == *id { count } else { 0 },
            ),
            _ => 0,
        }
    }

    pub fn unique_target_dependencies(&self) -> Vec<&Target> {
        let mut targets: Vec<&Target> = vec![];
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
pub struct Process<'a> {
    pub id: Uuid,
    pub name: String,
    pub primitives: Vec<Primitive<'a>>,
    pub dependencies: Vec<Dependency<'a>>,
    pub output: Vec<(&'a Target, usize)>,
    pub pois: Vec<&'a PointOfInterest>,
}

impl<'a> Process<'a> {
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
pub struct Spawn<'a> {
    pub id: Uuid,
    pub name: String,
    pub output: Option<(&'a Target, usize)>,
    pub pois: Vec<&'a PointOfInterest>,
}

impl<'a> Spawn<'a> {
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
pub struct Complete<'a> {
    pub id: Uuid,
    pub name: String,
    pub dependencies: Vec<Dependency<'a>>,
    pub pois: Vec<&'a PointOfInterest>,
}

impl<'a> Complete<'a> {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "".into(),
            dependencies: vec![],
            pois: vec![],
        }
    }
}
