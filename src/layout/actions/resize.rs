use rustwlc::{input, Point, ResizeEdge, Geometry,
              RESIZE_LEFT, RESIZE_RIGHT, RESIZE_TOP, RESIZE_BOTTOM,
              RESIZE_TOPLEFT, RESIZE_TOPRIGHT, RESIZE_BOTTOMLEFT, RESIZE_BOTTOMRIGHT,
};

use super::super::{Action, Direction, LayoutTree, TreeError};
use super::super::commands::{CommandResult};
use super::super::core::container::{ContainerType, MIN_SIZE};
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum ResizeErr {
    /// Expected the node associated with the UUID to be floating.
    ExpectedFloating(Uuid),
    /// Expected the node associated with the UUID to not be floating
    ExpectedNotFloating(Uuid)
}

impl LayoutTree {
    /// Updates pointer position for the node behind the id,
    /// in the same direction as the edge.
    pub fn update_pointer_pos(&self, id: Uuid, edge: ResizeEdge) -> CommandResult {
        let container = try!(self.lookup(id));
        let Geometry { mut origin, size } = container.get_geometry()
            .expect("Container had no geometry");
        drop(container);
        if edge.contains(RESIZE_TOPLEFT) {
            input::pointer::set_position(origin);
        } else if edge.contains(RESIZE_TOPRIGHT) {
            origin.x += size.w as i32;
            input::pointer::set_position(origin);
        } else if edge.contains(RESIZE_BOTTOMLEFT) {
            origin.y += size.h as i32;
            input::pointer::set_position(origin);
        } else if edge.contains(RESIZE_BOTTOMRIGHT) {
            origin.x += size.w as i32;
            origin.y += size.h as i32;
            input::pointer::set_position(origin);
        }
        Ok(())
    }

    /// Resizes a floating container. If the container was not floating, an Err is returned.
    pub fn resize_floating(&mut self, id: Uuid, edge: ResizeEdge, pointer: Point,
                           action: &mut Action) -> CommandResult {
        {
            let container = try!(self.lookup_mut(id));
            if !container.floating() {
                return Err(TreeError::Resize(ResizeErr::ExpectedFloating(container.get_id())))
            }
            match container.get_type() {
                ContainerType::View | ContainerType::Container => {},
                _ => return Err(TreeError::UuidWrongType(container.get_id(),
                                                        vec!(ContainerType::View)))
            }
            let geo = container.get_geometry()
                .expect("Could not get geometry of the container");

            let new_geo = calculate_resize(geo, edge, pointer, action.grab);
            action.grab = pointer;
            container.set_geometry(edge, new_geo);
        }
        self.update_pointer_pos(id, edge)
    }

    pub fn resize_tiled(&mut self, id: Uuid, edge: ResizeEdge, pointer: Point,
                        action: &mut Action) -> CommandResult {
        // There can be multiple directions we are moving in, e.g up and left.
        let parent_id = try!(self.parent_of(id)).get_id();
        let dirs_moving_in = Direction::from_edge(edge);
        let siblings: Vec<Uuid> = dirs_moving_in.iter()
            .map(|dir| self.sibling_in_dir(id, *dir))
            // TODO There MUST be a better way to do something like this...
            .filter(|thing| thing.is_ok())
            .map(|thing| thing.unwrap())
            .collect();

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
            let geo = container.get_geometry()
                .expect("Could not get geometry of the container");
            let new_geo = calculate_resize(geo, edge, pointer, action.grab);
            container.set_geometry(edge, new_geo);
        }
        //....update parent?
        {
            let container = try!(self.lookup_mut(parent_id));
            if container.floating() {
                return Err(TreeError::Resize(ResizeErr::ExpectedNotFloating(container.get_id())))
            }
            match container.get_type() {
                ContainerType::View | ContainerType::Container => {},
                _ => return Err(TreeError::UuidWrongType(container.get_id(),
                                                          vec!(ContainerType::View)))
            }
            let geo = container.get_geometry()
                .expect("Could not get geometry of the container");
            let new_geo = calculate_resize(geo, edge, pointer, action.grab);
            container.set_geometry(edge, new_geo);

        }
        // and now we mutate the siblings
        let reversed_dir: Vec<Direction> = dirs_moving_in.iter()
            .map(|dir| dir.reverse()).collect();
        let reversed_edge = Direction::to_edge(reversed_dir.as_slice());
        for sibling in siblings {
            // TODO Abstract, the only thing different from above
            // is the edge and the exact node we are doing the calc on
            let container = try!(self.lookup_mut(sibling));
            if container.floating() {
                return Err(TreeError::Resize(ResizeErr::ExpectedNotFloating(container.get_id())))
            }
            match container.get_type() {
                ContainerType::View | ContainerType::Container => {},
                _ => return Err(TreeError::UuidWrongType(container.get_id(),
                                                         vec!(ContainerType::View)))
            }
            let geo = container.get_geometry()
                .expect("Could not get geometry of the container");
            let new_geo = calculate_resize(geo, reversed_edge, pointer, action.grab);
            action.grab = pointer;
            container.set_geometry(reversed_edge, new_geo);
        }
        let node_ix = self.tree.lookup_id(id)
            .expect("Could not find node index for an id");
        let workspace_ix = try!(self.tree.ancestor_of_type(node_ix,
                                                           ContainerType::Workspace)
                                .map_err(|err| TreeError::PetGraph(err)));
        self.layout(workspace_ix);
        action.grab = pointer;
        self.update_pointer_pos(id, edge)
    }
}

/// Calculates what the new geometry is of a window.
/// Needs the geometry of the window, the edge direction the pointer is moving in,
/// the current position of the pointer, and the previous place the pointer was at.
fn calculate_resize(geo: Geometry, edge: ResizeEdge,
                    cur_pointer: Point, prev_pointer: Point) -> Geometry {
    let mut new_geo = geo.clone();
    let dx = cur_pointer.x - prev_pointer.x;
    let dy = cur_pointer.y - prev_pointer.y;
    if edge.contains(RESIZE_LEFT) {
        if dx < 0 {
            new_geo.size.w = geo.size.w.saturating_add(dx.abs() as u32);
        } else {
            new_geo.size.w = geo.size.w.saturating_sub(dx.abs() as u32);
        }
        new_geo.origin.x += dx;
    }
    else if edge.contains(RESIZE_RIGHT) {
        if dx < 0 {
            new_geo.size.w = geo.size.w.saturating_sub(dx.abs() as u32);
        } else {
            new_geo.size.w = geo.size.w.saturating_add(dx.abs() as u32);
        }
    }

    if edge.contains(RESIZE_TOP) {
        if dy < 0 {
            new_geo.size.h = geo.size.h.saturating_add(dy.abs() as u32);
        } else {
            new_geo.size.h = geo.size.h.saturating_sub(dy.abs() as u32);
        }
        new_geo.origin.y += dy;
    }
    else if edge.contains(RESIZE_BOTTOM) {
        if dy < 0 {
            new_geo.size.h = geo.size.h.saturating_sub(dy.abs() as u32);
        } else {
            new_geo.size.h = geo.size.h.saturating_add(dy.abs() as u32);
        }
    }

    if new_geo.size.w <= MIN_SIZE.w {
        new_geo.origin.x = geo.origin.x;
        new_geo.size.w = geo.size.w;
    }

    if new_geo.size.h <= MIN_SIZE.h {
        new_geo.origin.y = geo.origin.y;
        new_geo.size.h = geo.size.h;
    }
    new_geo
}
