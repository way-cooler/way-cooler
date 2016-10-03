use rustwlc::{Point, ResizeEdge, WlcView};

#[derive(Debug, Clone, Copy)]
pub struct Action {
    pub view: WlcView,
    pub grab: Point,
    pub edges: ResizeEdge
}
