use nalgebra::Vector3;

#[derive(Debug, encase::ShaderType)]
pub struct Query {
    pub origin: Vector3<f32>,
    pub dir_a: Vector3<f32>,
    pub dir_b: Vector3<f32>,
    pub resolution: u32,
}

#[derive(Debug, encase::ShaderType)]
pub struct Response {
    #[size(runtime)]
    pub heights: Vec<f32>,
}
