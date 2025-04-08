use nalgebra::{Point3, Vector3};

#[derive(Debug, encase::ShaderType)]
pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
}

impl Ray {
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>) -> Self {
        Self {
            origin: origin.coords,
            direction,
        }
    }
}
