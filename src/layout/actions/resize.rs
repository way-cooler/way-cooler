use rustwlc::{Point, Size, ResizeEdge,
              RESIZE_LEFT, RESIZE_RIGHT, RESIZE_TOP, RESIZE_BOTTOM};

static MIN_SIZE: Size = Size { w: 80u32, h: 40u32 };

use super::super::{Action, Direction, LayoutTree, TreeError};
use super::super::commands::{CommandResult};
use super::super::core::container::{ContainerType};
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum ResizeErr {
    /// Expected the node associated with the UUID to be floating.
    ExpectedFloating(Uuid),
    /// Expected the node associated with the UUID to not be floating
    ExpectedNotFloating(Uuid)
}

impl LayoutTree {
    /// Resizes a floating container. If the container was not floating, an Err is returned.
    pub fn resize_floating(&mut self, id: Uuid, edges: ResizeEdge, pointer: Point,
                           action: &mut Action) -> CommandResult {
        let container = try!(self.lookup_mut(id));
        if !container.floating() {
            return Err(TreeError::Resize(ResizeErr::ExpectedFloating(container.get_id())))
        }
        match container.get_type() {
            ContainerType::View | ContainerType::Container => {},
            _ => return Err(TreeError::UuidWrongType(container.get_id(),
                                                     vec!(ContainerType::View)))
        }
        let mut geo = container.get_geometry()
            .expect("Could not get geometry of the container");
        let mut new_geo = geo.clone();

        let grab = action.grab;
        action.grab = pointer;
        let dx = pointer.x - grab.x;
        let dy = pointer.y - grab.y;
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

        container.set_geometry(edges, geo);
        Ok(())
    }

    pub fn resize_tiled(&mut self, id: Uuid, edges: ResizeEdge, pointer: Point,
                        action: &mut Action) -> CommandResult {
        // There can be multiple directions we are moving in, e.g up and left.
        let dirs_moving_in = Direction::from_edge(edges);
        let siblings: Vec<Uuid> = dirs_moving_in.iter().map(|dir| self.sibling_in_dir(id, *dir)).collect();

        // Because we can't have multiple mutable references active in the tree at once,
        // first we modify the one the user is resizing and then adjust the siblings.
        {
            let container = try!(self.lookup_mut(id));
            if container.floating() {
                return Err(TreeError::Resize(ResizeErr::ExpectedNotFloating(container.get_id())))
            }
            match container.get_type() {
                ContainerType::View | ContainerType::Container => {},
                _ => return Err(TreeError::UuidWrongType(container.get_id(),
                                                        vec!(ContainerType::View)))
            }
            let mut cur_geo = container.get_geometry()
                .expect("Could not get geometry of the container");
            let mut cur_new_geo = cur_geo.clone();

            let grab = action.grab;
            action.grab = pointer;
            let dx = pointer.x - grab.x;
            let dy = pointer.y - grab.y;
            if edges.contains(RESIZE_LEFT) {
                if dx < 0 {
                    cur_new_geo.size.w += dx.abs() as u32;
                } else {
                    cur_new_geo.size.w -= dx.abs() as u32;
                }
                cur_new_geo.origin.x += dx;
            }
            else if edges.contains(RESIZE_RIGHT) {
                if dx < 0 {
                    cur_new_geo.size.w -= dx.abs() as u32;
                } else {
                    cur_new_geo.size.w += dx.abs() as u32;
                }
            }

            if edges.contains(RESIZE_TOP) {
                if dy < 0 {
                    cur_new_geo.size.h += dy.abs() as u32;
                } else {
                    cur_new_geo.size.h -= dy.abs() as u32;
                }
                cur_new_geo.origin.y += dy;
            }
            else if edges.contains(RESIZE_BOTTOM) {
                if dy < 0 {
                    cur_new_geo.size.h -= dy.abs() as u32;
                } else {
                    cur_new_geo.size.h += dy.abs() as u32;
                }
            }

            if cur_new_geo.size.w >= MIN_SIZE.w {
                cur_geo.origin.x = cur_new_geo.origin.x;
                cur_geo.size.w = cur_new_geo.size.w;
            }

            if cur_new_geo.size.h >= MIN_SIZE.h {
                cur_geo.origin.y = cur_new_geo.origin.y;
                cur_geo.size.h = cur_new_geo.size.h;
            }
        }
        // and now we mutate the siblings
        for sibling in siblings {
            // Do the same thing here as we do above, but reverse the directions
        }
        Ok(())
    }
}
