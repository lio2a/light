use nalgebra::{Point3, Vector3};

use super::Ray;

#[derive(Debug, encase::ShaderType)]
pub struct Intersection {
    pub ray: Ray,
    pub normal: Vector3<f32>,
    pub t: f32,
}

impl Intersection {
    pub fn point(&self) -> Point3<f32> {
        (self.ray.origin + self.ray.direction * self.t).into()
    }
}
