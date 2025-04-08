use crate::system::{Intersection, Ray};

#[derive(Debug, encase::ShaderType)]
pub struct Query {
    #[size(runtime)]
    pub rays: Vec<Ray>,
}

#[derive(Debug, Default, encase::ShaderType)]
pub struct Response {
    #[size(runtime)]
    pub intersections: Vec<Intersection>,
}
