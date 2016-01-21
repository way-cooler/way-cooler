extern crate rustwlc;

struct Data {
    handle: rustwlc::WLCHandle,
    grab: rustwlc::Point,
    drag_edge: rustwlc::ResizeEdge
}

pub data: mut 'static Data;

fn start_interactive_action(view: rustwlc::WLCHandle, origin: rustwlc::Point) -> bool
{
    data.handle = view;
    data.grab = origin;
    view.bring_to_front();
    return true;
}

fn main() {
    println!("Hello, world!");
    
}
