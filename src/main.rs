use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();
    
    // let event_loop = EventLoop::new().unwrap();
    // let mut engine = pollster::block_on(Engine::new(&event_loop));
    libpulsar::engine::render::init_renderer();
    
    // engine.run(event_loop);
}
