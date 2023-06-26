use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub enum TargetVariant {
    RawMaterial,
    Intermediate,
    Product,
    Singular
}

#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub id: Uuid,
    // pub target_variant: TargetVariant,
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

