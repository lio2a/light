use crate::compute::raytracing::RayTracingResult;

pub mod curvature;
pub mod radius;
pub mod thickness;

pub trait QueryParameter {
    type Output;

    fn query(&self, raytrace: &RayTracingResult) -> Self::Output;
}
