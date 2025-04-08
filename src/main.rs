#![feature(array_windows)]

use std::fmt::format;

mod compute;
// mod query;
mod system;
mod utils;

/// using MMGS as the units of measurement
#[tokio::main]
async fn main() {
    let system = system::System {
        object: system::Object {
            distance: 2.0,
            semi_diameter: 1.0,
            refractive_index: 1.0,
        },
        stop_index: 3u32,
        surfaces: vec![
            system::Surface {
                thickness: 1.05,
                refractive_index: 1.517,
                curvature: 1.0 / 7.3895,
                semi_diameter: 2.0,
            },
            system::Surface {
                thickness: 0.40,
                refractive_index: 1.649,
                curvature: 1.0 / -5.1784,
                semi_diameter: 2.0,
            },
            system::Surface {
                thickness: 10.55,
                refractive_index: 1.0,
                curvature: 1.0 / -16.2225,
                semi_diameter: 2.0,
            },
            system::Surface {
                thickness: 0.0,
                refractive_index: 1.0,
                curvature: 0.0,
                semi_diameter: 1.0,
            },
        ],
    };

    let origin = system.object.top();

    let directions = {
        let entry_limits = system.surface_edges(0).unwrap();

        (
            (entry_limits.0 - origin).normalize(),
            (entry_limits.1 - origin).normalize(),
        )
    };

    {
        let mut query = compute::fan::Query {
            origin: origin.coords,
            dir_a: directions.0,
            dir_b: directions.1,
            resolution: 4,
        };

        let mut i = 0;
        let result = loop {
            {
                let mut view = utils::View::new();
                let rays = (0..query.resolution)
                    .map(|i| {
                        system::Ray::new(
                            query.origin.into(),
                            query
                                .dir_a
                                .lerp(&query.dir_b, i as f32 / (query.resolution - 1) as f32)
                                .normalize(),
                        )
                    })
                    .collect::<Vec<_>>();

                {
                    let query = compute::raytracing::Query { rays };
                    let response = system.trace(&query).await.unwrap();

                    view.draw_intersections(&response.intersections);
                }

                view.draw_system(&system);
                view.finish();
                view.save(format!("../images/raytracing-{i}.svg")).unwrap();
            }

            let response = system.fan(&query).await.unwrap();

            let mut result = response
                .heights
                .array_windows::<2>()
                .enumerate()
                .filter_map(|(i, [a, b])| ((a * b) <= 0.0).then_some(i));

            if let Some(index) = result.next() {
                if result.next().is_some() {
                    panic!("Unexpected");
                }

                query = compute::fan::Query {
                    dir_a: query
                        .dir_a
                        .lerp(&query.dir_b, index as f32 / (query.resolution - 1) as f32)
                        .normalize(),
                    dir_b: query
                        .dir_a
                        .lerp(
                            &query.dir_b,
                            (index + 1) as f32 / (query.resolution - 1) as f32,
                        )
                        .normalize(),
                    ..query
                };

                let height = response.heights[index];
                if height.abs() < 1e-6 || i > 128 {
                    dbg!(i, height);
                    break query.dir_a;
                }
            } else {
                dbg!(i);
                break query.dir_a.lerp(&query.dir_b, 0.5).normalize();
            }

            i += 1;
        };

        {
            let mut view = utils::View::new();
            for i in 0..query.resolution {
                view.draw_ray(&system::Ray::new(
                    query.origin.into(),
                    query
                        .dir_a
                        .lerp(&query.dir_b, i as f32 / (query.resolution - 1) as f32)
                        .normalize(),
                ));
            }

            {
                let rays = vec![system::Ray::new(origin, result)];

                let query = compute::raytracing::Query { rays };
                let response = system.trace(&query).await.unwrap();

                view.draw_intersections(&response.intersections);

                dbg!(response.intersections.last().unwrap().point());
            }

            view.draw_system(&system);
            view.finish();
            view.save(format!("../images/raytracing-{i}.svg")).unwrap();
        }
    }
}
