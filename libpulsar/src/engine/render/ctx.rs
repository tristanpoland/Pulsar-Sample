
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use winit::window::Window;
use futures::executor::block_on;
#[derive(Debug, Error)]
pub enum ContextError {
    #[error("Failed to create WGPU surface: {0}")]
    SurfaceCreationFailure(#[from] wgpu::CreateSurfaceError),
}

/// This WGSL shader generates a cube procedurally and rotates it around the Y axis.
/// A uniform (u.time) is used as the rotation angle. After rotation, a simple
/// perspective projection is applied (dividing x,y by z) to produce clip-space coordinates.
const CUBE_SHADER: &str = r#"
// Uniform block containing time and the current aspect ratio.
// (The extra two padding floats ensure 16-byte alignment.)
struct Uniforms {
    time: f32,
    aspect: f32,
    padding0: f32,
    padding1: f32,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;

// Rotation around the X axis (tilt)
fn rotationX(angle: f32) -> mat3x3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat3x3<f32>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, c, -s),
        vec3<f32>(0.0, s,  c)
    );
}

// Rotation around the Y axis (spinning)
fn rotationY(angle: f32) -> mat3x3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat3x3<f32>(
        vec3<f32>( c, 0.0, s),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(-s, 0.0, c)
    );
}

// Define the eight cube vertices (a unit cube centered at the origin)
const offsets: array<vec3<f32>, 8> = array<vec3<f32>, 8>(
    vec3<f32>(-0.5, -0.5, -0.5),
    vec3<f32>( 0.5, -0.5, -0.5),
    vec3<f32>( 0.5,  0.5, -0.5),
    vec3<f32>(-0.5,  0.5, -0.5),
    vec3<f32>(-0.5, -0.5,  0.5),
    vec3<f32>( 0.5, -0.5,  0.5),
    vec3<f32>( 0.5,  0.5,  0.5),
    vec3<f32>(-0.5,  0.5,  0.5)
);

// Indices for the 12 triangles that form the six faces of the cube.
const indices: array<u32, 36> = array<u32, 36>(
    0, 1, 2, 2, 3, 0, // Front face
    4, 5, 6, 6, 7, 4, // Back face
    0, 1, 5, 5, 4, 0, // Bottom face
    2, 3, 7, 7, 6, 2, // Top face
    0, 3, 7, 7, 4, 0, // Left face
    1, 2, 6, 6, 5, 1  // Right face
);

/// Vertex shader
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> @builtin(position) vec4<f32> {
    // Look up the vertex position using the index array.
    let pos = offsets[indices[vid]];

    // Create a dynamic rotation:
    // rotY rotates over time, while rotX is fixed (45° tilt).
    let rotY = rotationY(u.time );
    let rotX = rotationX(u.time / 1.5 ); 

    // Combine the rotations: first rotate around Y, then tilt with X.
    var transformedPos = rotX * (rotY * pos);

    // Translate the cube along the Z axis so it appears in front of the camera.
    transformedPos = transformedPos + vec3<f32>(0.0, 0.0, 2.0);

    // Apply a simple perspective projection:
    // Divide x and y by z, and correct the x coordinate for the aspect ratio.
    let projected = vec2<f32>(
        (transformedPos.x / transformedPos.z) * (1.0 / u.aspect),
        transformedPos.y / transformedPos.z
    );
    return vec4<f32>(projected, 0.0, 1.0);
}

/// Fragment shader
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    // The color changes over time. We use sine functions to create smooth oscillation.
    // sin(u.time) oscillates between -1.0 and 1.0.
    // Multiplying by 0.5 and adding 0.5 scales the range to 0.0 .. 1.0.
    let red   = 0.5 * sin(u.time)           + 0.5;
    let green = 0.5 * sin(u.time + 2.094)     + 0.5; // 2.094 ≈ 2π/3 phase shift
    let blue  = 0.5 * sin(u.time + 4.188)     + 0.5; // 4.188 ≈ 4π/3 phase shift

    // The resulting vec4 is our final color with full opacity.
    return vec4<f32>(red, green, blue, 1.0);
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
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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
        block_on(Self::new(window))
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        let (width, height) = new_size;
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn draw(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        // Compute the aspect ratio from the current surface configuration.
        let aspect = self.surface_config.width as f32 / self.surface_config.height as f32;
        // Pack time and aspect into four floats (with two padding zeros).
        let uniform_data = [elapsed, aspect, 0.0, 0.0];
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&uniform_data));

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