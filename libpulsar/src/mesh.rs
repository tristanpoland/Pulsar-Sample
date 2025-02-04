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
    pub depth_texture: (wgpu::Texture, wgpu::TextureView),
}

impl Mesh {
    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        (texture, view)
    }

    pub fn cube(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
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

        let depth_texture = Self::create_depth_texture(device, config);

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            depth_texture,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.depth_texture = Self::create_depth_texture(device, config);
    }
}