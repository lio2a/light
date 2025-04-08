use super::QueryParameter;

pub struct Radius {
    pub index: usize,
}

impl QueryParameter for Radius {
    type Output = f32;

    fn query(&self, raytrace: &super::RayTracingResult) -> Self::Output {
        raytrace.system.surfaces[self.index].curvature.recip()
    }
}
