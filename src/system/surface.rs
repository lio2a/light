use nalgebra::Point2;

#[derive(Debug, Clone, encase::ShaderType)]
pub struct Surface {
    pub thickness: f32,
    pub refractive_index: f32,
    pub curvature: f32,
    pub semi_diameter: f32,
}

impl Surface {
    /// Returns the z-coordinate of the surface at the given point.
    pub fn z(&self, point: Point2<f32>) -> f32 {
        let radius = self.curvature.recip();

        radius - (radius * radius - point.x * point.x - point.y * point.y).sqrt() * radius.signum()
    }

    pub fn sagitta(&self) -> f32 {
        let radius = self.curvature.recip();

        radius.signum()
            * (radius.abs() - (radius * radius - self.semi_diameter * self.semi_diameter).sqrt())
    }
}

impl Default for Surface {
    fn default() -> Self {
        Self {
            thickness: 0.0,
            refractive_index: 1.0,
            curvature: 0.0,
            semi_diameter: 0.0,
        }
    }
}
