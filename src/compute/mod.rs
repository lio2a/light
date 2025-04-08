pub mod fan;
pub mod raytracing;

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
}

impl Gpu {
    pub async fn new() -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance.request_adapter(&Default::default()).await.unwrap();
        let features = adapter.features();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Compute Device"),
                    required_features: features,
                    ..Default::default()
                },
                None,
            )
            .await?;

        let bind_group_layouts = vec![
            // Group 0 -- System Bind Group Layout
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("System Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            }),
            // Group 1 -- Raytracing Bind Group Layout
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Raytracing Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            }),
        ];

        Ok(Self {
            device,
            queue,
            bind_group_layouts,
        })
    }
}
