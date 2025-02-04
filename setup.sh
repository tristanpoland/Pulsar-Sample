#!/bin/bash

# Create the main project
cargo new MyProject
cd MyProject

# Create necessary directories
mkdir -p src/{assets,objects,plugins}
mkdir -p libpulsar/src/{engine/plugins,render,resources,math}

# Create main project Cargo.toml
cat > Cargo.toml << 'EOL'
[package]
name = "my_project"
version = "0.1.0"
edition = "2021"

[dependencies]
libpulsar = { path = "libpulsar" }
wgpu = "0.18"
winit = "0.29"
glam = "0.24"
pollster = "0.3"
bytemuck = { version = "1.14", features = ["derive"] }
log = "0.4"
env_logger = "0.10"
gltf = "1.3"
futures = "0.3"
EOL

# Create libpulsar Cargo.toml
cat > libpulsar/Cargo.toml << 'EOL'
[package]
name = "libpulsar"
version = "0.1.0"
edition = "2021"

[dependencies]
wgpu = "0.18"
winit = "0.29"
glam = "0.24"
pollster = "0.3"
bytemuck = { version = "1.14", features = ["derive"] }
log = "0.4"
gltf = "1.3"
futures = "0.3"
EOL

# Create main.rs
cat > src/main.rs << 'EOL'
use libpulsar::engine::Engine;
use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();
    let mut engine = pollster::block_on(Engine::new(&event_loop));
    
    engine.run(event_loop);
}
EOL

# Create lib.rs for libpulsar
cat > libpulsar/src/lib.rs << 'EOL'
pub mod engine;
pub mod renderer;
pub mod mesh;
pub mod camera;

pub use engine::Engine;
EOL

# Create camera.rs
cat > libpulsar/src/camera.rs << 'EOL'
use glam::{Vec3, Mat4};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
    }
}

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(position: Vec3, aspect: f32) -> Self {
        Self {
            position,
            target: Vec3::ZERO,
            up: Vec3::Y,
            aspect,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.position, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);
        proj * view
    }
}
EOL

# Create mesh.rs
cat > libpulsar/src/mesh.rs << 'EOL'
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl Mesh {
    pub fn cube(device: &wgpu::Device) -> Self {
        let vertices = [
            // Front face
            Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 0.0, 0.0] },
            Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 0.0, 0.0] },
            
            // Back face
            Vertex { position: [-0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0] },
            Vertex { position: [-0.5,  0.5, -0.5], color: [0.0, 1.0, 0.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], color: [0.0, 1.0, 0.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0] },
            
            // Top face
            Vertex { position: [-0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0] },
            Vertex { position: [-0.5,  0.5,  0.5], color: [0.0, 0.0, 1.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], color: [0.0, 0.0, 1.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0] },
            
            // Bottom face
            Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 1.0, 0.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 1.0, 0.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 1.0, 0.0] },
            Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 1.0, 0.0] },
            
            // Right face
            Vertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 0.0, 1.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], color: [1.0, 0.0, 1.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 0.0, 1.0] },
            
            // Left face
            Vertex { position: [-0.5, -0.5, -0.5], color: [0.0, 1.0, 1.0] },
            Vertex { position: [-0.5, -0.5,  0.5], color: [0.0, 1.0, 1.0] },
            Vertex { position: [-0.5,  0.5,  0.5], color: [0.0, 1.0, 1.0] },
            Vertex { position: [-0.5,  0.5, -0.5], color: [0.0, 1.0, 1.0] },
        ];

        let indices: &[u16] = &[
            0,  1,  2,  2,  3,  0,  // front
            4,  5,  6,  6,  7,  4,  // back
            8,  9,  10, 10, 11, 8,  // top
            12, 13, 14, 14, 15, 12, // bottom
            16, 17, 18, 18, 19, 16, // right
            20, 21, 22, 22, 23, 20, // left
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
}
EOL

# Create renderer.rs
cat > libpulsar/src/renderer.rs << 'EOL'
use wgpu::util::DeviceExt;
use crate::{camera::CameraUniform, mesh::Vertex};

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
}

impl Renderer {
    pub async fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform::new()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            pipeline,
            camera_bind_group,
            camera_buffer,
        }
    }

    pub fn render(
        &self,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh: &crate::mesh::Mesh,
    ) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, camera_uniform: &CameraUniform) {
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[*camera_uniform]));
    }
}
EOL

# Create shader.wgsl
cat > libpulsar/src/shader.wgsl << 'EOL'
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
EOL

# Create engine/mod.rs
cat > libpulsar/src/engine/mod.rs << 'EOL'
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};
use glam::Vec3;
use std::sync::Arc;

use crate::{
    camera::{Camera, CameraUniform},
    mesh::Mesh,
    renderer::Renderer,
};

pub struct Engine {
    window: Arc<Window>,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
    mesh: Mesh,
    camera: Camera,
    camera_uniform: CameraUniform,
    depth_texture: (wgpu::Texture, wgpu::TextureView),
}

impl Engine {
    pub async fn new(event_loop: &EventLoop<()>) -> Self {
        let window = WindowBuilder::new()
            .with_title("3D Engine")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
            .build(event_loop)
            .unwrap();
        let window = Arc::new(window);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let renderer = Renderer::new(&device, &config).await;
        let mesh = Mesh::cube(&device);

        // Create camera
        let camera = Camera::new(Vec3::new(2.0, 2.0, 2.0), config.width as f32 / config.height as f32);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        // Create depth texture
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            window,
            surface,
            device,
            queue,
            config,
            renderer,
            mesh,
            camera,
            camera_uniform,
            depth_texture: (depth_texture, depth_view),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            
            // Recreate depth texture
            self.depth_texture = {
                let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Depth Texture"),
                    size: wgpu::Extent3d {
                        width: self.config.width,
                        height: self.config.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth32Float,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                (texture, view)
            };

            // Update camera aspect ratio
            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
            self.camera_uniform.update_view_proj(&self.camera);
            self.renderer.update_camera(&self.queue, &self.camera_uniform);
        }
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        let _ = event_loop.run(move |event, target| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => match event {
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::Resized(physical_size) => {
                        self.resize(*physical_size);
                    }
                    _ => {}
                },
                Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                    match self.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => self.resize(self.window.inner_size()),
                        Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                Event::MainEventsCleared => {
                    self.window.request_redraw();
                }
                _ => {}
            }
        });
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.render(
            &view,
            &self.depth_texture.1,
            &self.device,
            &self.queue,
            &self.mesh,
        )?;

        output.present();
        Ok(())
    }
}
EOL

# Create basic assets directory
mkdir -p src/assets
touch src/assets/logo.png
touch src/assets/character.glb

echo "3D Engine implementation completed successfully!"
echo "Run 'cargo run' to start the engine and see a colored cube!"