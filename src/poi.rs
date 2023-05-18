use nalgebra::Vector3;

#[derive(Clone,Debug)]
pub struct PointOfInterest {
    pub id: String,
    pub name: String,
    pub position: Vector3<f64>
}

impl PointOfInterest {
    pub fn new(id: String, name: String, x: f64, y: f64, z: f64) -> Self {
        Self { id, name, position: Vector3::new(x, y, z)}
    }
}