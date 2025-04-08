mod intersection;
mod object;
mod ray;
mod surface;

use bytemuck::Contiguous;
use encase::ShaderSize;
pub use intersection::*;
use nalgebra::Point3;
pub use object::*;
pub use ray::*;
pub use surface::*;
use wgpu::{include_wgsl, util::DeviceExt};

use crate::compute;

#[derive(Debug, Default, encase::ShaderType)]
pub struct System {
    pub object: Object,
    pub stop_index: u32,
    #[size(runtime)]
    pub surfaces: Vec<Surface>,
}

impl System {
    /// Returns the entry limits of the first surface.
    pub fn surface_edges(&self, index: usize) -> Option<(Point3<f32>, Point3<f32>)> {
        let surface = self.surfaces.get(index)?;
        let sagitta = surface.sagitta();

        Some((
            Point3::new(0.0, surface.semi_diameter, sagitta),
            Point3::new(0.0, -surface.semi_diameter, sagitta),
        ))
    }

    pub async fn trace(
        &self,
        query: &compute::raytracing::Query,
    ) -> anyhow::Result<compute::raytracing::Response> {
        let n_intersections = self.surfaces.len() * query.rays.len();
        let gpu = compute::Gpu::new().await?;

        let module = gpu
            .device
            .create_shader_module(include_wgsl!("../shaders/raytracing.wgsl"));

        let mut system_bytes_buffer = encase::StorageBuffer::new(Vec::<u8>::new());
        let mut query_bytes_buffer = encase::StorageBuffer::new(Vec::<u8>::new());

        system_bytes_buffer.write(&self)?;
        query_bytes_buffer.write(&query)?;

        let system_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("System Buffer"),
                contents: system_bytes_buffer.as_ref(),
                usage: wgpu::BufferUsages::MAP_READ
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
            });

        let query_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Query Buffer"),
                contents: query_bytes_buffer.as_ref(),
                usage: wgpu::BufferUsages::MAP_READ
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
            });

        let result_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Result Buffer"),
            size: (n_intersections as u64) * Intersection::SHADER_SIZE.into_integer(),
            usage: wgpu::BufferUsages::MAP_READ
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        // Input bind group
        let bind_group0_layout = &gpu.bind_group_layouts[0];
        let bind_group0 = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Input Bind Group"),
            layout: bind_group0_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: system_buffer.as_entire_binding(),
            }],
        });

        // Output bind group
        let bind_group1_layout = &gpu.bind_group_layouts[1];
        let bind_group1 = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ray tracing Bind Group"),
            layout: bind_group1_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: query_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: result_buffer.as_entire_binding(),
                },
            ],
        });

        // Compute pipeline
        let compute_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[bind_group0_layout, bind_group1_layout],
                    push_constant_ranges: &[],
                });

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &module,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        // Command encoder
        let mut encoder = gpu.device.create_command_encoder(&Default::default());

        // Compute pass
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group0, &[]);
            pass.set_bind_group(1, &bind_group1, &[]);
            pass.dispatch_workgroups(query.rays.len() as _, 1, 1);
        }

        gpu.queue.submit(Some(encoder.finish()));

        {
            let buffer_slice = result_buffer.slice(..);
            // let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);
            let (sender, receiver) = tokio::sync::oneshot::channel();

            buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

            gpu.device.poll(wgpu::Maintain::Wait);

            receiver.await??;

            let data_raw = &*buffer_slice.get_mapped_range();
            let result_bytes_buffer = encase::StorageBuffer::new(data_raw);

            let response = result_bytes_buffer.create()?;

            Ok(response)
        }
    }

    pub async fn fan(&self, query: &compute::fan::Query) -> anyhow::Result<compute::fan::Response> {
        let gpu = compute::Gpu::new().await?;

        let module = gpu
            .device
            .create_shader_module(include_wgsl!("../shaders/fan.wgsl"));

        let mut system_bytes_buffer = encase::StorageBuffer::new(Vec::<u8>::new());
        let mut query_bytes_buffer = encase::StorageBuffer::new(Vec::<u8>::new());

        system_bytes_buffer.write(&self)?;
        query_bytes_buffer.write(&query)?;

        let system_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("System Buffer"),
                contents: system_bytes_buffer.as_ref(),
                usage: wgpu::BufferUsages::MAP_READ
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
            });

        let query_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Query Buffer"),
                contents: query_bytes_buffer.as_ref(),
                usage: wgpu::BufferUsages::MAP_READ
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
            });

        let result_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Result Buffer"),
            size: (query.resolution as u64) * f32::SHADER_SIZE.into_integer(),
            usage: wgpu::BufferUsages::MAP_READ
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        // Input bind group
        let bind_group0_layout = &gpu.bind_group_layouts[0];
        let bind_group0 = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Input Bind Group"),
            layout: bind_group0_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: system_buffer.as_entire_binding(),
            }],
        });

        // Output bind group
        let bind_group1_layout = &gpu.bind_group_layouts[1];
        let bind_group1 = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fan Bind Group"),
            layout: bind_group1_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: query_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: result_buffer.as_entire_binding(),
                },
            ],
        });

        // Compute pipeline
        let compute_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[bind_group0_layout, bind_group1_layout],
                    push_constant_ranges: &[],
                });

        let compute_pipeline =
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Pipeline"),
                    layout: Some(&compute_pipeline_layout),
                    module: &module,
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                });

        // Command encoder
        let mut encoder = gpu.device.create_command_encoder(&Default::default());

        // Compute pass
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&compute_pipeline);
            pass.set_bind_group(0, &bind_group0, &[]);
            pass.set_bind_group(1, &bind_group1, &[]);
            pass.dispatch_workgroups(query.resolution, 1, 1);
        }

        gpu.queue.submit(Some(encoder.finish()));

        {
            let buffer_slice = result_buffer.slice(..);
            // let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);
            let (sender, receiver) = tokio::sync::oneshot::channel();

            buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

            gpu.device.poll(wgpu::Maintain::Wait);

            receiver.await??;

            let data_raw = &*buffer_slice.get_mapped_range();
            let result_bytes_buffer = encase::StorageBuffer::new(data_raw);

            let response = result_bytes_buffer.create()?;

            Ok(response)
        }
    }

    pub async fn find_chief_ray(&self) -> anyhow::Result<Ray> {
        let origin = self.object.top();

        let directions = {
            let entry_limits = self.surface_edges(0).unwrap();

            (
                (entry_limits.0 - origin).normalize(),
                (entry_limits.1 - origin).normalize(),
            )
        };

        let mut query = compute::fan::Query {
            origin: origin.coords,
            dir_a: directions.0,
            dir_b: directions.1,
            resolution: 1024,
        };

        let mut i = 0;
        let direction = loop {
            let response = self.fan(&query).await?;

            let mut result = response
                .heights
                .array_windows::<2>()
                .enumerate()
                .filter_map(|(i, [a, b])| {
                    if a < b {
                        *a <= 0.0 && 0.0 < *b
                    } else {
                        *b <= 0.0 && 0.0 < *a
                    }
                    .then_some(i)
                });

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

        Ok(Ray::new(origin, direction))
    }
}
