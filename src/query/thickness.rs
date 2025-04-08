use nalgebra::Point2;

use super::QueryParameter;

pub struct Thickness {
    pub from: usize,
    pub to: usize,
    pub origin: Point2<f32>,
}

impl QueryParameter for Thickness {
    type Output = f32;

    fn query(&self, raytrace: &super::RayTracingResult) -> Self::Output {
        raytrace.system.surfaces[self.from..self.to]
            .iter()
            .fold(0.0, |acc, surface| {
                acc + surface.thickness - surface.z(self.origin)
            })
    }
}
