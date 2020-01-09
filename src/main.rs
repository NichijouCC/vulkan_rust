mod base;
use arrayvec::ArrayVec;

use base::{
    hal_state::HalState, local_state::LocalState, user_input::UserInput, winit_state::WinitState,
};

fn main() {
    let mut winit_state = WinitState::default();
    let mut hal_state = match HalState::new(&winit_state.window) {
        Ok(state) => state,
        Err(e) => panic!(e),
    };    
    
    let (frame_width, frame_height) = winit_state
        .window
        .get_inner_size()
        .map(|logical| logical.into())
        .unwrap_or((0.0, 0.0));
    let mut local_state = LocalState {
        frame_width,
        frame_height,
        mouse_x: 0.0,
        mouse_y: 0.0,
    };
    // println!("Hello, world!");   
    loop {
        let inputs = UserInput::poll_events_loop(&mut winit_state.events_loop);
        if inputs.end_requested {
            break;
        }
        local_state.update_from_input(inputs);
        hal_state.render();
    }
}
