use nalgebra::Point3;

#[derive(Debug, Default, encase::ShaderType)]
pub struct Object {
    pub distance: f32,
    pub semi_diameter: f32,
    pub refractive_index: f32,
}

impl Object {
    pub fn top(&self) -> Point3<f32> {
        Point3::new(0.0, self.semi_diameter, -self.distance)
    }
}
