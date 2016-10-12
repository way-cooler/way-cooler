use rustwlc::{Point, Size, ResizeEdge,
              RESIZE_LEFT, RESIZE_RIGHT, RESIZE_TOP, RESIZE_BOTTOM};

static MIN_SIZE: Size = Size { w: 80u32, h: 40u32 };

use super::super::{Action, LayoutTree, TreeError};
use super::super::commands::{CommandResult};
use super::super::core::container::{ContainerType, Handle};
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum ResizeErr {
    /// Expected the node associated with the UUID to be floating.
    ExpectedFloating(Uuid)
}

impl LayoutTree {
    /// Resizes a floating container. If the container was not floating, an Err is returned.
    pub fn resize_floating(&mut self, id: Uuid, edges: ResizeEdge, point: Point,
                           action: &mut Action) -> CommandResult {
        let grab = action.grab;
        //let edges = action.edges;
        action.grab = point;
        let container = try!(self.lookup(id));
        if !container.floating() {
            return Err(TreeError::Resize(ResizeErr::ExpectedFloating(container.get_id())))
        }
        if container.get_type() != ContainerType::View {
            return Err(TreeError::UuidWrongType(container.get_id(),
                                                vec!(ContainerType::View)))
        }
        let handle = match container.get_handle() {
            Some(Handle::View(view)) => view,
            _ => unreachable!()
        };
        let mut geo = handle.get_geometry()
            .expect("Could not get geometry of a view");
        let mut new_geo = geo.clone();

        let dx = point.x - grab.x;
        let dy = point.y - grab.y;
        if edges.contains(RESIZE_LEFT) {
            if dx < 0 {
                new_geo.size.w += dx.abs() as u32;
            } else {
                new_geo.size.w -= dx.abs() as u32;
            }
            new_geo.origin.x += dx;
        }
        else if edges.contains(RESIZE_RIGHT) {
            if dx < 0 {
                new_geo.size.w -= dx.abs() as u32;
            } else {
                new_geo.size.w += dx.abs() as u32;
            }
        }

        if edges.contains(RESIZE_TOP) {
            if dy < 0 {
                new_geo.size.h += dy.abs() as u32;
            } else {
                new_geo.size.h -= dy.abs() as u32;
            }
            new_geo.origin.y += dy;
        }
        else if edges.contains(RESIZE_BOTTOM) {
            if dy < 0 {
                new_geo.size.h -= dy.abs() as u32;
            } else {
                new_geo.size.h += dy.abs() as u32;
            }
        }

        if new_geo.size.w >= MIN_SIZE.w {
            geo.origin.x = new_geo.origin.x;
            geo.size.w = new_geo.size.w;
        }

        if new_geo.size.h >= MIN_SIZE.h {
            geo.origin.y = new_geo.origin.y;
            geo.size.h = new_geo.size.h;
        }

        handle.set_geometry(edges, geo);
        Ok(())
    }
}
