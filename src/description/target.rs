use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub id: Uuid,
    pub name: String,
    pub size: f64,
    pub weight: f64
}

impl Target  {
    pub fn new(name: String, size: f64, weight: f64) -> Target  {
        Target {
            id: Uuid::new_v4(),
            name,
            size,
            weight
        }
    }
}