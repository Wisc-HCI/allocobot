use crate::description::agent::Agent;
use crate::description::poi::PointOfInterest;
use crate::description::primitive::Primitive;
use crate::description::target::Target;
use crate::description::task::Task;
use crate::description::rating::Rating;
use crate::petri::net::PetriNet;
use enum_tag::EnumTag;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: Uuid,
    pub name: String,
    pub tasks: HashMap<Uuid, Task>,
    pub primitives: HashMap<Uuid, Primitive>,
    pub points_of_interest: HashMap<Uuid, PointOfInterest>,
    pub agents: HashMap<Uuid, Agent>,
    pub targets: HashMap<Uuid, Target>,
    pub basic_net: Option<PetriNet>,
    pub agent_net: Option<PetriNet>,
    pub poi_net: Option<PetriNet>,
    pub cost_net: Option<PetriNet>,
}

impl Job {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            tasks: HashMap::new(),
            primitives: HashMap::new(),
            points_of_interest: HashMap::new(),
            agents: HashMap::new(),
            targets: HashMap::new(),
            basic_net: None,
            agent_net: None,
            poi_net: None,
            cost_net: None,
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.insert(task.id, task);
    }

    pub fn add_primitive(&mut self, primitive: Primitive) {
        self.primitives.insert(primitive.id(), primitive);
    }

    pub fn add_point_of_interest(&mut self, poi: PointOfInterest) {
        self.points_of_interest.insert(poi.id(), poi);
    }

    pub fn add_agent(&mut self, agent: Agent) {
        self.agents.insert(agent.id(), agent);
    }

    pub fn add_target(&mut self, target: Target) {
        self.targets.insert(target.id(), target);
    }

    pub fn create_task(&mut self, name: String) -> Uuid {
        let task = Task::new_empty(name);
        let uuid = task.id;
        self.add_task(task);
        uuid
    }

    pub fn create_standing_point_of_interest(
        &mut self,
        name: String,
        x: f64,
        y: f64,
        z: f64,
        variability: Option<Rating>,
        structure: Option<Rating>
    ) -> Uuid {
        let poi = PointOfInterest::new_standing(name, x, y, z, variability, structure);
        let uuid = poi.id();
        self.add_point_of_interest(poi);
        uuid
    }

    pub fn create_hand_point_of_interest(&mut self, name: String, x: f64, y: f64, z: f64, variability: Option<Rating>,
        structure: Option<Rating>) -> Uuid {
        let poi = PointOfInterest::new_hand(name, x, y, z, variability, structure);
        let uuid = poi.id();
        self.add_point_of_interest(poi);
        uuid
    }

    pub fn create_robot_agent(
        &mut self,
        name: String,
        reach: f64,        // meters
        payload: f64,      // kg
        agility: Rating,      // rating
        speed: f64,        // m/s
        precision: f64,    // m (repeatability)
        sensing: Rating,      // rating
        mobile_speed: f64, // m/s (zero if not mobile)
    ) -> Uuid {
        let agent = Agent::new_robot(
            name,
            reach,
            payload,
            agility,
            speed,
            precision,
            sensing,
            mobile_speed,
        );
        let uuid = agent.id();
        self.add_agent(agent);
        uuid
    }

    pub fn create_human_agent(
        &mut self,
        name: String,
        age: f64,
        acromial_height: f64,
        height: f64,
        reach: f64,
        weight: f64,
        skill: Rating
    ) -> Uuid {
        let agent = Agent::new_human(
            name,
            age,
            acromial_height,
            height,
            reach,
            weight,
            skill
        );
        let uuid = agent.id();
        self.add_agent(agent);
        uuid
    } 

    pub fn create_precursor_target(&mut self, name: String, size: f64, weight: f64) -> Uuid {
        let target = Target::new_precursor(name, size, weight);
        let uuid = target.id();
        self.add_target(target);
        uuid
    }

    pub fn create_intermediate_target(&mut self, name: String, size: f64, weight: f64) -> Uuid {
        let target = Target::new_intermediate(name, size, weight);
        let uuid = target.id();
        self.add_target(target);
        uuid
    }

    pub fn create_product_target(&mut self, name: String, size: f64, weight: f64) -> Uuid {
        let target = Target::new_product(name, size, weight);
        let uuid = target.id();
        self.add_target(target);
        uuid
    }

    pub fn create_reusable_target(&mut self, name: String, size: f64, weight: f64) -> Uuid {
        let target = Target::new_reusable(name, size, weight);
        let uuid = target.id();
        self.add_target(target);
        uuid
    }

    pub fn add_task_dependency(&mut self, task: Uuid, target: Uuid, count: usize) {
        match self.tasks.get_mut(&task) {
            Some(task_obj) => task_obj.add_dependency(&target, count),
            None => {}
        }
    }

    pub fn add_task_output(&mut self, task: Uuid, target: Uuid, count: usize) {
        match self.tasks.get_mut(&task) {
            Some(task_obj) => task_obj.add_output(&target, count),
            None => {}
        }
    }

    pub fn add_task_point_of_interest(&mut self, task: Uuid, poi: Uuid) {
        match self.tasks.get_mut(&task) {
            Some(task_obj) => task_obj.add_point_of_interest(&poi),
            None => {}
        }
    }

    pub fn add_task_reusable(&mut self, task: Uuid, target: Uuid, count: usize) {
        match self.tasks.get_mut(&task) {
            Some(task_obj) => task_obj.add_reusable(&target, count),
            None => {}
        }
    }

    pub fn add_task_primitive(&mut self, task: Uuid, primitive: Primitive) {
        match self.tasks.get_mut(&task) {
            Some(task_obj) => {
                task_obj.add_primitive(primitive.id());
                self.primitives.insert(primitive.id(), primitive);
            }
            None => {}
        }
    }

    pub fn create_petri_nets(&mut self) {
        self.basic_net = Some(self.create_basic_net());
        self.agent_net = Some(self.create_agent_net());
        self.poi_net = Some(self.create_poi_net());
        self.cost_net = Some(self.create_cost_net());
    }

    pub fn create_agent_net(&mut self) -> PetriNet {
        if !self.basic_net.is_some() {
            self.basic_net = Some(self.create_basic_net());
        }
        self.compute_agent_from_basic()
    }

    pub fn create_poi_net(&mut self) -> PetriNet {
        if !self.agent_net.is_some() {
            self.agent_net = Some(self.create_agent_net());
        }
        self.compute_poi_from_agent()
    }

    pub fn create_cost_net(&mut self) -> PetriNet {
        if !self.poi_net.is_some() {
            self.poi_net = Some(self.create_poi_net());
        }
        self.compute_cost_from_poi()
    }
}

pub type PrimitiveTag = <Primitive as EnumTag>::Tag;
