/* Structures */
struct Surface {
    thickness: f32,
    refractive_index: f32,
    curvature: f32,
    semi_diameter: f32,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct Intersection {
    ray: Ray,
    normal: vec3<f32>,
    t: f32,
}

struct Object {
    distance: f32,
    semi_diameter: f32,
    refractive_index: f32,
}

struct System {
    object: Object,
    stop_index: u32,
    surfaces: array<Surface>,
}

/* Ray tracing */
struct Query {
    rays: array<Ray>,
}

struct Result {
    intersections: array<Intersection>,
}

/* Global variables */
/* Group 0 -- Surfaces */
@group(0)
@binding(0)
var<storage, read> system: System;

/* Group 1 -- Ray Tracing */
@group(1)
@binding(0)
var<storage, read> query: Query;

@group(1)
@binding(1)
var<storage, read_write> result: Result;

/* Functions */
fn refract(dir: vec3<f32>, normal: vec3<f32>, mu: f32) -> vec3<f32> {
    let a = dot(normal, dir);
    return normalize(sqrt(1.0 - mu * mu * (1.0 - a * a)) * normal + mu * (dir - a * normal));
}

fn intersect_with_sphere(surface: Surface, ray: Ray, z: f32, n0: f32) -> Intersection {
    let nan = bitcast<f32>(0x7FC00000u);

    let radius = 1.0 / surface.curvature;
    let center = vec3<f32>(0.0, 0.0, z + radius);

    let a = dot(ray.direction, ray.direction); // 1.0 because ray.direction is normalized
    let b = 2.0 * dot(ray.direction, ray.origin - center);
    let c = dot(ray.origin - center, ray.origin - center) - radius * radius;

    let delta = sqrt(b * b - 4.0 * a * c);

    let t1 = (-b - delta) / (2.0 * a);
    let t2 = (-b + delta) / (2.0 * a);

    let s = sign(radius);
    let t = select(
        t1, // if r < 0
        t2, // if r > 0
        radius < 0.0,
    );
    let normal = normalize(ray.direction * t + ray.origin - center) * s;

    return Intersection(ray, normal, t);
}

fn intersect_with_plane(surface: Surface, ray: Ray, z: f32, n0: f32) -> Intersection {
    let t = (z - ray.origin.z) / ray.direction.z;
    let normal = vec3(0.0, 0.0, -1.0);

    return Intersection(ray, normal, t);
}

/* Entry points */
@compute
@workgroup_size(1, 1, 1)
fn main(
    @builtin(global_invocation_id)
    global_id: vec3<u32>
) {
    let index = global_id.x;
    let n_surfaces = arrayLength(&system.surfaces);
    let offset = index * n_surfaces;

    var ray = query.rays[index];
    var n0 = system.object.refractive_index;
    var z = 0.0;

    for (var i = 0u; i < n_surfaces; i++) {
        let surface = system.surfaces[i];

        var intersection: Intersection;

        if (surface.curvature == 0.0) {
            intersection = intersect_with_plane(surface, ray, z, n0);
        } else {
            intersection = intersect_with_sphere(surface, ray, z, n0);
        }

        result.intersections[offset + i] = intersection;

        ray = Ray(
            ray.direction * intersection.t + ray.origin, // origin
            refract(
                ray.direction,
                -intersection.normal,
                n0 / surface.refractive_index
            ), // direction
        );

        z += surface.thickness;
        n0 = surface.refractive_index;
    }
}
