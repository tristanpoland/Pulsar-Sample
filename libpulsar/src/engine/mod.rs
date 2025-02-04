use std::sync::Arc;

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};
use glam::Vec3;

use crate::{
    camera::{Camera, CameraUniform},
    mesh::Mesh,
    renderer::Renderer,
};

pub struct Engine<'window> {
    window: Arc<Window>,
    instance: wgpu::Instance,
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
    mesh: Mesh,
    camera: Camera,
    camera_uniform: CameraUniform,
    depth_texture: (wgpu::Texture, wgpu::TextureView),
}

impl<'window> Engine<'window> {
    pub fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let surface = instance.create_surface(&*window).unwrap();
        let window: Arc<Window> = Arc::clone(&window);
        
        let adapter = futures::executor::block_on(
            instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
        ).unwrap();

        let (device, queue) = futures::executor::block_on(
            adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
        ).unwrap();

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
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let renderer = futures::executor::block_on(Renderer::new(&device, &config));
        let mesh = Mesh::cube(&device, &config);

        let camera = Camera::new(Vec3::new(2.0, 2.0, 2.0), config.width as f32 / config.height as f32);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let depth_texture = Self::create_depth_texture(&device, &config);

        Self {
            window,
            instance,
            surface,
            device,
            queue,
            config,
            renderer,
            mesh,
            camera,
            camera_uniform,
            depth_texture,
        }
    }

    fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
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
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    pub fn create(event_loop: &EventLoop<()>) -> Self {
        let window = WindowBuilder::new()
            .with_title("3D Engine")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
            .build(event_loop)
            .unwrap();

        Self::new(window.into())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            
            // Update camera aspect ratio
            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
            self.camera_uniform.update_view_proj(&self.camera);
            
            // Recreate depth texture with new size
            self.depth_texture = Self::create_depth_texture(&self.device, &self.config);
            
            // Configure surface with new size
            self.surface.configure(&self.device, &self.config);
            
            // Update renderer if needed
            self.renderer.resize(&self.device, &self.config);
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Add any update logic here, such as camera movement
        // self.camera.update(dt);
        self.camera_uniform.update_view_proj(&self.camera);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.render(
            &self.device,
            &self.queue,
            &self.surface,
            &self.camera_uniform,
            &self.mesh,
        )
    }
}

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    let mut engine = Engine::create(&event_loop);
    
    let mut last_frame_time = std::time::Instant::now();
    
    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == engine.window.id() => {
                target.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                window_id,
            } if window_id == engine.window.id() => {
                engine.resize(new_size);
            }
            Event::AboutToWait => {
                // Calculate delta time
                let current_time = std::time::Instant::now();
                let dt = (current_time - last_frame_time).as_secs_f32();
                last_frame_time = current_time;

                // Update engine state
                engine.update(dt);
                
                // Request a redraw
                engine.window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id,
            } if window_id == engine.window.id() => {
                match engine.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => engine.resize(engine.window.inner_size()),
                    Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => {}
        }
    });
}