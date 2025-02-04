use std::borrow::Cow;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use winit::window::Window;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("Failed to create WGPU surface: {0}")]
    SurfaceCreationFailure(#[from] wgpu::CreateSurfaceError),
}

/// This WGSL shader generates a cube procedurally and rotates it around the Y axis.
/// A uniform (u.time) is used as the rotation angle. After rotation, a simple
/// perspective projection is applied (dividing x,y by z) to produce clip-space coordinates.
const CUBE_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let vertices = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );
    
    var output: VertexOutput;
    output.position = vec4<f32>(vertices[vertex_index], 0.0, 1.0);
    output.uv = vertices[vertex_index] * 0.5 + 0.5;
    return output;
}

fn drawSpaceship(uv: vec2<f32>) -> vec4<f32> {
    // Center and scale UV coordinates
    var p = uv - vec2<f32>(0.5);
    p = p * 2.0;
    
    // Ship body shape
    var ship = 0.0;
    
    // Main body (triangle)
    let bodyDist = abs(p.x) + p.y - 0.2;
    if (bodyDist < 0.1) {
        ship = 1.0;
    }
    
    // Wings
    let wingDist = abs(abs(p.x) - 0.3) + abs(p.y + 0.1) - 0.1;
    if (wingDist < 0.1) {
        ship = 1.0;
    }
    
    // Cockpit
    let cockpitDist = length(p - vec2<f32>(0.0, -0.1)) - 0.1;
    if (cockpitDist < 0.05) {
        ship = 0.8;  // Slightly transparent for cockpit
    }
    
    // Engine glow
    let engineDist = length(p - vec2<f32>(0.0, 0.3)) - 0.15;
    let glow = smoothstep(0.2, 0.0, engineDist);
    
    // Colors
    var color = vec3<f32>(0.0);
    if (ship > 0.0) {
        // Ship body color (silver-white)
        color = vec3<f32>(0.8, 0.8, 0.9);
    }
    
    // Add engine glow (orange-red)
    color = color + vec3<f32>(1.0, 0.3, 0.1) * glow;
    
    return vec4<f32>(color, ship + glow * 0.5);
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    return drawSpaceship(uv);
}
"#;

pub struct WgpuCtx<'window> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'window>,
    surface_config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    start_time: Instant,
}

impl<'window> WgpuCtx<'window> {
    pub async fn new(window: Arc<Window>) -> Result<WgpuCtx<'window>, ContextError> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to obtain render adapter");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .expect("Failed to create rendering device");

        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        let surface_config = surface.get_default_config(&adapter, width, height).unwrap();
        surface.configure(&device, &surface_config);

        // Create a uniform buffer (16 bytes to satisfy alignment requirements)
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create the shader module from the inline WGSL shader.
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Cube Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(CUBE_SHADER)),
        });

        // Create a bind group layout for the uniform.
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(16),
                },
                count: None,
            }],
        });

        // Create the pipeline layout.
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cube Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // TODO: add proper vertex buffer
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Cube Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(WgpuCtx {
            device,
            queue,
            surface,
            surface_config,
            adapter,
            render_pipeline,
            uniform_buffer,
            start_time: Instant::now(),
        })
    }

    pub fn new_blocking(window: Arc<Window>) -> Result<WgpuCtx<'window>, ContextError> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { WgpuCtx::new(window).await })
        })
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        let (width, height) = new_size;
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn draw(&mut self) {
        // Update the uniform buffer with the elapsed time.
        let elapsed = self.start_time.elapsed().as_secs_f32();
        // Pack into 4 floats (pad to 16 bytes)
        let time_data = [elapsed, 0.0, 0.0, 0.0];
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&time_data));

        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Failed to get surface texture");
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Cube Command Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Cube Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            // Create a bind group on the fly for the uniform.
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Uniform Bind Group (per draw)"),
                layout: &self.render_pipeline.get_bind_group_layout(0),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                }],
            });
            render_pass.set_bind_group(0, &bind_group, &[]);
            // Draw 36 vertices (6 faces × 6 vertices)
            render_pass.draw(0..36, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}
