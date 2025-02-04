use libpulsar::engine::Engine;
use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();
    let mut engine = pollster::block_on(Engine::new(&event_loop));
    
    engine.run(event_loop);
}
