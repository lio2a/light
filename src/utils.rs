use svg::Node;

use crate::system;

const COLORS: &[&str] = &[
    "red", "orange", "yellow", "lime", "blue", "indigo", "violet",
];

pub struct View {
    pub document: svg::Document,
    min_z: f32,
    max_z: f32,
    max_y: f32,
}

impl View {
    pub fn new() -> Self {
        Self {
            document: svg::Document::new()
                .add(
                    svg::node::element::Rectangle::new()
                        .set("fill", "black")
                        .set("x", -1000)
                        .set("y", -1000)
                        .set("width", 2000)
                        .set("height", 2000),
                )
                .add(
                    svg::node::element::Line::new()
                        .set("x1", -1000.0)
                        .set("y1", 0.0)
                        .set("x2", 1000.0)
                        .set("y2", 0.0)
                        .set("stroke", "gray")
                        .set("stroke-width", 0.025)
                        .set("stroke-dasharray", "0.25 0.125 0.125 0.125 0.125 0.125")
                        .set("stroke-linecap", "round"),
                ),
            min_z: 0.0,
            max_z: 0.0,
            max_y: 0.0,
        }
    }

    pub fn draw_surface(&mut self, surface: &system::Surface, z: f32) {
        let radius = surface.curvature.recip();

        self.min_z = self.min_z.min(z);
        self.max_z = self.max_z.max(z + surface.thickness);
        self.max_y = self.max_y.max(surface.semi_diameter);

        if radius.is_finite() {
            self.document.append(
                svg::node::element::Circle::new()
                    .set("cx", z + radius)
                    .set("cy", 0.0)
                    .set("r", radius.abs())
                    .surface()
                    .extension(),
            );
            self.document.append(
                svg::node::element::Path::new()
                    .set("d", {
                        let sagitta = surface.sagitta();

                        svg::node::element::path::Data::new()
                            .move_to((z + sagitta, -surface.semi_diameter))
                            .elliptical_arc_to((
                                radius.abs(),
                                radius.abs(),
                                0.0,
                                0,
                                if surface.curvature.signum() > 0.0 {
                                    0
                                } else {
                                    1
                                },
                                z + sagitta,
                                surface.semi_diameter,
                            ))
                    })
                    .surface(),
            );
        } else {
            self.document.append(
                svg::node::element::Line::new()
                    .set("x1", z)
                    .set("y1", -1000.0)
                    .set("x2", z)
                    .set("y2", 1000.0)
                    .surface()
                    .extension(),
            );
            self.document.append(
                svg::node::element::Line::new()
                    .set("x1", z)
                    .set("y1", -surface.semi_diameter)
                    .set("x2", z)
                    .set("y2", surface.semi_diameter)
                    .surface(),
            );
        }
    }

    pub fn draw_object(&mut self, object: &system::Object) {
        self.min_z = self.min_z.min(-object.distance);
        self.max_z = self.max_z.max(-object.distance);
        self.max_y = self.max_y.max(object.semi_diameter);

        self.document.append(
            svg::node::element::Line::new()
                .set("x1", -object.distance)
                .set("y1", 0.0)
                .set("x2", -object.distance)
                .set("y2", object.semi_diameter)
                .set("stroke", "yellow")
                .set("stroke-width", 0.025)
                .set("stroke-linecap", "round"),
        );
    }

    pub fn draw_intersection(&mut self, intersection: &system::Intersection) {
        let point = intersection.ray.origin + intersection.t * intersection.ray.direction;

        self.document.append(
            svg::node::element::Line::new()
                .set("x1", intersection.ray.origin.z)
                .set("y1", intersection.ray.origin.y)
                .set("x2", point.z)
                .set("y2", point.y)
                .set("stroke", "cyan")
                .set("stroke-width", 0.025)
                .set("stroke-linecap", "round"),
        );
        self.document.append(
            svg::node::element::Line::new()
                .set("x1", point.z)
                .set("y1", point.y)
                .set("x2", point.z + intersection.normal.z * 0.25)
                .set("y2", point.y + intersection.normal.y * 0.25)
                .normal(),
        );
    }

    pub fn draw_intersections(&mut self, intersections: &[system::Intersection]) {
        for intersection in intersections {
            self.draw_intersection(intersection);
        }
    }

    pub fn draw_ray(&mut self, ray: &system::Ray) {
        self.document.append(
            svg::node::element::Line::new()
                .set("x1", ray.origin.z)
                .set("y1", ray.origin.y)
                .set("x2", ray.origin.z + ray.direction.z)
                .set("y2", ray.origin.y + ray.direction.y)
                .set("stroke", "lime")
                .set("stroke-width", 0.025)
                .set("stroke-linecap", "round"),
        );
    }

    pub fn draw_system(&mut self, system: &system::System) {
        self.draw_object(&system.object);
        let mut z = 0.0;

        for surface in system.surfaces.iter() {
            self.draw_surface(surface, z);
            z += surface.thickness;
        }
    }

    pub fn finish(&mut self) {
        let width = self.max_z - self.min_z;
        let height = 2.0 * self.max_y;

        self.document.assign(
            "viewBox",
            (
                self.min_z - 0.1 * width,
                -self.max_y - 0.1 * height,
                1.2 * width,
                1.2 * height,
            ),
        );
        self.document.assign("width", 16.0 * width);
        self.document.assign("height", 32.0 * height);
    }

    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<(), std::io::Error> {
        svg::save(path, &self.document)
    }
}

trait AuxNode: svg::Node + Sized {
    fn extension(self) -> Self;
    fn surface(self) -> Self;
    fn normal(self) -> Self;
}

impl<T> AuxNode for T
where
    T: svg::Node + Sized,
{
    fn extension(mut self) -> Self {
        self.assign("stroke-width", 0.025);
        self.assign("stroke-dasharray", "0.25 0.25");
        self.assign("stroke-linecap", "round");
        self.assign("opacity", 0.25);

        self
    }

    fn surface(mut self) -> Self {
        self.assign("fill", "none");
        self.assign("stroke", "white");
        self.assign("stroke-width", 0.025);
        self.assign("stroke-linecap", "round");

        self
    }

    fn normal(mut self) -> Self {
        self.assign("stroke", "magenta");
        self.assign("stroke-width", 0.025);
        self.assign("stroke-linecap", "round");

        self
    }
}
