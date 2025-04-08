use super::QueryParameter;

pub struct Curvature {
    pub index: usize,
}

impl QueryParameter for Curvature {
    type Output = f32;

    fn query(&self, raytrace: &super::RayTracingResult) -> Self::Output {
        raytrace.system.surfaces[self.index].curvature
    }
}
