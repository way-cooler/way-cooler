//! Container types

use std::fmt;

use uuid::Uuid;
use rustwlc::handle::{WlcView, WlcOutput};
use rustwlc::{Geometry, ResizeEdge, Point, Size,
              VIEW_FULLSCREEN, VIEW_BIT_MODAL};

use ::render::{Renderable, Drawable};
use ::layout::commands::CommandResult;
use super::borders::{Borders, ViewDraw, ContainerDraw};
use super::tree::TreeError;
use super::bar::Bar;
use super::background::MaybeBackground;

pub static MIN_SIZE: Size = Size { w: 80u32, h: 40u32 };

/// A handle to either a view or output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handle {
    View(WlcView),
    Output(WlcOutput)
}

impl From<WlcView> for Handle {
    fn from(view: WlcView) -> Handle {
        Handle::View(view)
    }
}

impl From<WlcOutput> for Handle {
    fn from(output: WlcOutput) -> Handle {
        Handle::Output(output)
    }
}

/// Types of containers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerType {
    /// Root container, only one exists
    Root,
    /// WlcOutput/Monitor
    Output,
    /// A workspace
    Workspace,
    /// A Container, houses views and other containers
    Container,
    /// A view (window)
    View
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerErr {
    /// A bad operation on the container type.
    /// The human readable string provides more context.
    BadOperationOn(ContainerType, String)
}


impl ContainerType {
    /// Whether this container can be used as the parent of another
    pub fn can_have_child(self, other: ContainerType) -> bool {
        use self::ContainerType::*;
        match self {
            Root => other == Output,
            Output => other == Workspace,
            Workspace => other == Container,
            Container => other == Container || other == View,
            View => false
        }
    }
}

/// Layout mode for a container
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Horizontal,
    Vertical,
    Tabbed,
    Stacked
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Layout::Horizontal => "horizontal",
            Layout::Vertical => "vertical",
            Layout::Tabbed => "tabbed",
            Layout::Stacked => "stacked",
        })
    }
}

/// Represents an item in the container tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Container {
    /// Root node of the container
    Root(Uuid),
    /// Output
    Output {
        /// Handle to the wlc
        handle: WlcOutput,
        /// Optional background for the output
        background: Option<MaybeBackground>,
        /// Optional bar for the output
        bar: Option<Bar>,
        /// UUID associated with container, client program can use container
        id: Uuid,
    },
    /// Workspace
    Workspace {
        /// Name of the workspace
        name: String,
        /// The geometry of the workspace on the screen.
        /// Might be different if there is e.g a bar present
        geometry: Geometry,
        /// `Vec` of all children that are fullscreen.
        /// This is used to disable certain features while there is a fullscreen
        /// (e.g: focus switching, resizing, and moving containers)
        fullscreen_c: Vec<Uuid>,
        /// UUID associated with container, client program can use container
        id: Uuid,
    },
    /// Container
    Container {
        /// How the container is layed out
        layout: Layout,
        /// If the container is floating
        floating: bool,
        /// If the container is fullscreen
        fullscreen: bool,
        /// The handle for the output the container resides in.
        /// Used for border drawing
        output_handle: WlcOutput,
        /// The apparent geometry, as seen by the user.
        /// This is the size of the container including borders.
        /// Used in `update_container_geo_for_borders` and in border drawing
        /// for containers.
        apparent_geometry: Geometry,
        /// The geometry of the container, relative to the parent container
        /// This is used for tiling children containers and is the "real"
        /// geometry for the container.
        geometry: Geometry,
        /// UUID associated with container, client program can use container
        id: Uuid,
        /// The border drawn to the screen
        borders: Option<Borders>,
    },
    /// View or window
    View {
        /// The wlc handle to the view
        handle: WlcView,
        /// Whether this view is floating
        floating: bool,
        /// Effective geometry. This is the size of the container including
        /// borders and gaps. It does _not_ change when an app becomes
        /// fullscreen. E.g to get the fullscreen size use `handle.get_geometry`
        effective_geometry: Geometry,
        /// UUID associated with container, client program can use container
        id: Uuid,
        /// The border drawn to the screen
        borders: Option<Borders>,
    }
}

impl Container {
    /// Creates a new root container.
    pub fn new_root() -> Container {
        Container::Root(Uuid::new_v4())
    }
    /// Creates a new output container with the given output
    pub fn new_output(handle: WlcOutput) -> Container {
        Container::Output {
            handle: handle,
            background: None,
            bar: None,
            id: Uuid::new_v4()
        }
    }

    /// Creates a new workspace container with the given name and size.
    /// Usually the size is the same as the output it resides on,
    /// unless there is a bar or something.
    pub fn new_workspace(name: String, geometry: Geometry) -> Container {
        Container::Workspace {
            name: name,
            geometry: geometry,
            fullscreen_c: Vec::new(),
            id: Uuid::new_v4()
        }
    }

    /// Creates a new container
    pub fn new_container(geometry: Geometry,
                         output_handle: WlcOutput,
                         borders: Option<Borders>) -> Container {
        Container::Container {
            layout: Layout::Horizontal,
            floating: false,
            fullscreen: false,
            output_handle,
            apparent_geometry: geometry,
            geometry,
            id: Uuid::new_v4(),
            borders
        }
    }

    /// Creates a new view container with the given handle
    pub fn new_view(handle: WlcView, borders: Option<Borders>) -> Container {
        let geometry = handle.get_geometry()
            .expect("View had no geometry");
        Container::View {
            handle: handle,
            floating: false,
            effective_geometry: geometry,
            id: Uuid::new_v4(),
            borders: borders
        }
    }

    /// Sets the visibility of this container
    pub fn set_visibility(&mut self, visibility: bool) {
        let mask = if visibility { 1 } else { 0 };
        if let Ok(handle) = self.get_handle() {
            match handle {
                Handle::View(view) => {
                    view.set_mask(mask)
                },
                _ => {},
            }
        }
    }

    /// Gets the type of this container
    pub fn get_type(&self) -> ContainerType {
        match *self {
            Container::Root(_) => ContainerType::Root,
            Container::Output { .. } => ContainerType::Output,
            Container::Workspace { .. } => ContainerType::Workspace,
            Container::Container { .. } => ContainerType::Container,
            Container::View { .. } => ContainerType::View
        }
    }

    /// Gets the view handle of the view container, if this is a view container
    pub fn get_handle(&self) -> Result<Handle, ContainerErr> {
        match *self {
            Container::View { ref handle, ..} => Ok(Handle::View(handle.clone())),
            Container::Output { ref handle, .. } => Ok(Handle::Output(handle.clone())),
            ref other => Err(ContainerErr::BadOperationOn(
                other.get_type(),
                "Only views and outputs have handles".into()))
        }
    }

    /// Gets the name of the workspace, if this container is a workspace.
    pub fn get_name(&self) -> Option<&str> {
        match *self {
            Container::Workspace { ref name, ..} => Some(name),
            _ => None
        }
    }

    /// Gets the geometry of the container, if the container has one.
    /// Root: Returns None
    /// Workspace/Output: Size is the size of the screen, origin is just 0,0
    /// Container/View: Size is the size of the container,
    /// origin is the coordinates relative to the parent container.
    pub fn get_geometry(&self) -> Option<Geometry> {
        match *self {
            Container::Root(_)  => None,
            Container::Output { ref handle, ref bar, .. } => {
                let mut resolution = handle.get_resolution()
                    .expect("Couldn't get output resolution");
                let mut origin = Point { x: 0, y: 0 };
                if let Some(handle) = bar.as_ref().map(|bar| **bar) {
                    let bar_g = handle.get_geometry()
                        .expect("Bar had no geometry");
                    let Size { h, .. } = bar_g.size;
                    // TODO Allow bars on the horizontal side
                    // This is for bottom
                    //resolution.h = resolution.h.saturating_sub(h);
                    origin.y += h as i32;
                    resolution.h = resolution.h.saturating_sub(h)
                }
                Some(Geometry {
                    origin: origin,
                    size: resolution
                })
            },
            Container::Workspace { geometry, .. } |
            Container::Container { geometry, .. } => Some(geometry),
            Container::View { effective_geometry, .. } => {
                Some(effective_geometry)
            },
        }
    }

    /// Gets the actual geometry for a `WlcView`, `Container`, or `WlcOutput`.
    ///
    /// Unlike `get_geometry`, this does not account for borders/gaps,
    /// and instead is just a thin wrapper around
    /// `handle.get_geometry`/`container.geometry`/`handle.get_resolution`.
    ///
    /// Most of the time you want `get_geometry`, as you should account for the
    /// borders, gaps, and top bar.
    ///
    /// For non-`View`/`Container`/`Output` containers, this always returns `None`
    pub fn get_actual_geometry(&self) -> Option<Geometry> {
        match *self {
            Container::View { handle, .. } => handle.get_geometry(),
            Container::Container { geometry, .. } => Some(geometry),
            Container::Output { handle, .. } => {
                handle.get_resolution()
                    .map(|size|
                         Geometry {
                             origin: Point { x: 0, y: 0 },
                             size: size
                         })
            },
            _ => None
        }
    }

    /// Sets the geometry behind the container. Panics if container is root.
    ///
    /// For view you need to set the appropriate edges (which can be empty).
    /// If you are not intending to set the geometry of a view, simply pass `ResizeEdge::empty()`
    pub fn set_geometry(&mut self, edges: ResizeEdge, geo: Geometry) {
        match *self {
            Container::Root(_) =>
                panic!("Tried to set the geometry of the root!"),
            Container::Output { ref handle, .. } => {
                handle.set_resolution(geo.size, 1);
            },
            Container::Workspace { ref mut geometry, .. } |
            Container::Container { ref mut geometry, .. } => {
                *geometry = geo;
            },
            Container::View { ref handle, ref mut effective_geometry, .. } => {
                handle.set_geometry(edges, geo);
                *effective_geometry = geo;
            }
        }
    }

    pub fn set_layout(&mut self, new_layout: Layout) -> Result<(), ContainerErr>{
        match *self {
            Container::Container { ref mut layout, ref mut borders, .. } => {
                *layout = new_layout;
                if let Some(ref mut borders) = borders.as_mut() {
                    borders.title = format!("{:?} container", new_layout);
                }
                Ok(())
            },
            ref other => Err(ContainerErr::BadOperationOn(
                other.get_type(),
                format!("Can only set the layout of a container, not {:?}",
                        other)))
        }
    }

    pub fn get_layout(&self) -> Result<Layout, ContainerErr> {
        match *self {
            Container::Container { layout, .. } => Ok(layout),
            ref other => Err(ContainerErr::BadOperationOn(
                other.get_type(),
                "Only containers have a layout!".into()
            ))
        }
    }

    pub fn get_id(&self) -> Uuid {
        match *self {
            Container::Root(id) | Container::Output { id, .. } |
            Container::Workspace { id, .. } | Container::Container { id, .. } |
            Container::View { id, .. } => {
                id
            }
        }
    }

    pub fn floating(&self) -> bool {
        match *self {
            Container::View { floating, .. } | Container::Container { floating, .. } => floating,
            Container::Workspace { .. } | Container::Output { .. } | Container::Root(_) => false
        }
    }


    // TODO Make these set_* functions that can fail return a proper error type.

    /// If not set on a view or container, error is returned telling what
    /// container type that this function was (incorrectly) called on.
    pub fn set_floating(&mut self, val: bool) -> Result<ContainerType, ContainerType> {
        let c_type = self.get_type();
        let mut v_g;
        match *self {
            Container::View { handle, ref mut floating, .. } => {
                *floating = val;
                // And now we update the geometry, if necessary.
                v_g = handle.get_geometry() .expect("View had no geometry");
                // Make it the min size
                if v_g.size.w < MIN_SIZE.w {
                    v_g.size.w = MIN_SIZE.w;
                }
                if v_g.size.h < MIN_SIZE.h {
                    v_g.size.h = MIN_SIZE.h;
                }
                // if modal, center it if in the top left.
                let put_in_center = handle.get_type().contains(VIEW_BIT_MODAL) ||
                    handle.get_parent() != WlcView::root();
                if put_in_center {
                    if v_g.origin.x == 0 && v_g.origin.y == 0 {
                        let output = handle.get_output();
                        let res = output.get_resolution()
                            .expect("Output had no resolution");
                        v_g.origin.x = (res.w / 2 - v_g.size.w / 2) as i32;
                        v_g.origin.y = (res.h / 2 - v_g.size.h / 2) as i32;
                    }
                }
                // sorry, we would return here but borrow checker is fussy.
            },
            Container::Container { ref mut floating, .. } => {
                *floating = val;
                return Ok(c_type)
            },
            _ => {
                return Err(c_type)
            }
        }
        self.set_geometry(ResizeEdge::empty(), v_g);
        Ok(c_type)
    }

    /// Sets the fullscreen flag on the container to the specified value.
    ///
    /// If called on a non View/Container, then returns an Err with the wrong type.
    pub fn set_fullscreen(&mut self, val: bool) -> Result<(), ContainerType> {
        let c_type = self.get_type();
        let floating = self.floating();
        match *self {
            Container::View { handle, effective_geometry, .. } => {
                handle.set_state(VIEW_FULLSCREEN, val);
                if !val {
                    let new_geometry;
                    if floating {
                        let output_size = handle.get_output().get_resolution()
                            .expect("output had no resolution");
                        new_geometry = Geometry {
                            size: Size {
                                h: output_size.h / 2,
                                w: output_size.w / 2
                            },
                            origin: Point {
                                x: (output_size.w / 2 - output_size.w / 4) as i32 ,
                                y: (output_size.h / 2 - output_size.h / 4) as i32
                            }
                        };
                    } else {
                        new_geometry = effective_geometry;
                    }
                    handle.set_geometry(ResizeEdge::empty(), new_geometry)
                }
                Ok(())
            },
            Container::Container { ref mut fullscreen, .. } => {
                *fullscreen = val;
                Ok(())
            },
            _ => Err(c_type)
        }
    }

    /// Determines if a container is fullscreen.
    ///
    /// Workspaces, Outputs, and the Root are never fullscreen.
    pub fn fullscreen(&self) -> bool {
        match *self {
            Container::View { handle, .. } => {
                handle.get_state().intersects(VIEW_FULLSCREEN)
            },
            Container::Container { fullscreen, .. } => fullscreen,
            _ => false
        }
    }

    /// Updates the workspace (`self`) that the `id` resides in to reflect
    /// whether the container with the `id` is fullscreen (`toggle`).
    ///
    /// If called with a non-workspace an Err is returned with
    /// the incorrect type.
    pub fn update_fullscreen_c(&mut self, id: Uuid, toggle: bool)
                               -> Result<(), ContainerType> {
        let c_type = self.get_type();
        match *self {
            Container::Workspace { ref mut fullscreen_c, .. } => {
                if !toggle {
                    match fullscreen_c.iter().position(|c_id| *c_id == id) {
                        Some(index) => { fullscreen_c.remove(index); },
                        None => {}
                    }
                } else {
                    fullscreen_c.push(id);
                }
                Ok(())
            },
            _ => Err(c_type)
        }
    }

    /// If the container is a workspace, returns the children in the workspace that
    /// are fullscreen. The last child is the one visible to the user.
    ///
    /// Computes in O(1) time.
    ///
    /// If the container is not a workspace, None is returned.
    pub fn fullscreen_c(&self) -> Option<&Vec<Uuid>> {
        match *self {
            Container::Workspace { ref fullscreen_c, .. } =>
                Some(fullscreen_c),
            _ => None
        }
    }

    /// Gets the name of the container.
    ///
    /// Container::Root: returns simply the string "Root Container"
    /// Container::Output: The name of the output
    /// Container::Workspace: The name of the workspace
    /// Container::Container: Layout style (e.g horizontal)
    /// Container::View: The name taken from `WlcView`
    pub fn name(&self) -> String {
        match *self {
            Container::Root(_)  => "Root Container".into(),
            Container::Output { handle, .. } => {
                handle.get_name()
            },
            Container::Workspace { ref name, .. } => name.clone(),
            Container::Container { ref borders, layout, .. } => {
                borders.as_ref().map(|b| b.title().into())
                    .unwrap_or_else(|| format!("{:?}", layout))
            },
            Container::View { handle, ..} => {
                Container::get_title(handle)
            }
        }
    }

    pub fn set_name(&mut self, new_name: String) {
        let c_type = self.get_type();
        match *self {
            Container::View { ref mut borders, .. } |
            Container::Container { ref mut borders, .. } => {
                borders.as_mut().map(|b| b.title = new_name);
            },
            Container::Workspace { ref mut name, .. } => {
                *name = new_name;
            },
            _ => warn!("Tried to set name of {:?} to {}",
                       c_type, new_name)
        }
    }


    pub fn render_borders(&mut self) {
        match *self {
            Container::View { ref mut borders, .. } |
            Container::Container { ref mut borders, .. } => {
                if let Some(borders) = borders.as_mut() {
                    borders.render();
                }
            },
            _ => panic!("Tried to render a non-view / non-container")
        }
    }

    /// Draws the borders around the container.
    ///
    /// If the type of the container is not `Container` or `View`, then an
    /// error is returned.
    ///
    /// If there are no borders associated with the `Container`/`View`,
    /// then argument returns `Ok` but nothing is done.
    pub fn draw_borders(&mut self) -> Result<(), ContainerErr> {
        // TODO Eventually, we should use an enum to choose which way to draw the
        // border, but for now this will do.
        match *self {
            Container::View { ref mut borders, handle, .. } => {
                if let Some(mut borders_) = borders.take() {
                    let geometry = handle.get_geometry()
                        .expect("View had no geometry");
                    if borders_.geometry != geometry {
                        if let Some(new_borders) = borders_.reallocate_buffer(geometry) {
                            borders_ = new_borders;
                        } else {
                            return Ok(())
                        }
                    }
                    *borders = ViewDraw::new(borders_.enable_cairo().unwrap())
                        .draw(geometry).ok();
                }
                Ok(())
            },
            Container::Container { layout,
                                   ref mut borders,
                                   apparent_geometry: mut geometry, .. } => {
                if let Some(mut borders_) = borders.take() {
                    // update the title of the borders

                    borders_.title = match layout {
                        Layout::Tabbed | Layout::Stacked => {
                            // Already filled in from `layout.rs`
                            borders_.title
                        },
                        _ => format!("{:?} container", layout)
                    };
                    if borders_.geometry != geometry {
                        // NOTE This is a hack to work around how borders work...
                        // This fixes a bug where the title border extends too
                        // far to the left, because it allocates space for a
                        // side border that is never drawn. And thus it
                        // over-draws.
                        let thickness = Borders::thickness() as i32;
                        geometry.origin.x += thickness / 2;
                        geometry.size.w = geometry.size.w.saturating_sub(thickness as u32);
                        if let Some(new_borders) = borders_.reallocate_buffer(geometry) {
                            borders_ = new_borders;
                        } else {
                            return Ok(())
                        }
                    }
                    *borders = ContainerDraw::new(borders_.enable_cairo().unwrap())
                        .draw(geometry).ok();
                }
                Ok(())
            },
            ref other => Err(ContainerErr::BadOperationOn(
                other.get_type(), "Tried to render a non-view/container".into()))
        }
    }

    /// Resizes the border buffer to fit within this geometry, if the
    /// `View`/`Container` has a border wrapping it.
    ///
    /// # Panics
    /// Panics on non-`View`/`Container`s
    pub fn resize_borders(&mut self, geo: Geometry) {
        match *self {
            Container::View { ref mut borders, ..}  => {
                if let Some(borders_) = borders.take() {
                    *borders = borders_.reallocate_buffer(geo)
                }
            },
            Container::Container { output_handle,
                                   ref mut borders, ..} => {
                *borders = borders.take().and_then(|b| b.reallocate_buffer(geo))
                    .or_else(|| Borders::new(geo, output_handle));
            }
            ref container => {
                error!("Tried to resize border to {:#?} on {:#?}", geo, container);
                panic!("Expected a View/Container, got a different type")
            }
        }
    }

    /// Set the border color on a view or container to be the active color.
    ///
    /// If called on a non-view/container, returns an appropriate error.
    pub fn active_border_color(&mut self) -> CommandResult {
        let c_type = self.get_type();
        match *self {
            Container::View { ref mut borders, .. } |
            Container::Container { ref mut borders, .. }=> {
                if let Some(borders_) = borders.as_mut() {
                    let color = Borders::active_color();
                    let title_color = Borders::active_title_color();
                    let title_font_color = Borders::active_title_font_color();
                    borders_.set_color(color);
                    borders_.set_title_color(title_color);
                    borders_.set_title_font_color(title_font_color);
                }
                Ok(())
            },
            _ => Err(TreeError::Container(
                ContainerErr::BadOperationOn(c_type,
                                               "active_border_color".into())))
        }
    }

    /// Clears the border color on a view or container.
    ///
    /// If called on a non-view/container, returns an appropriate error.
    pub fn clear_border_color(&mut self) -> CommandResult {
        let c_type = self.get_type();
        match *self {
            Container::View { ref mut borders, .. } |
            Container::Container { ref mut borders, .. }=> {
                if let Some(borders_) = borders.as_mut() {
                    borders_.set_color(None);
                    borders_.set_title_color(None);
                    borders_.set_title_font_color(None);
                }
                Ok(())
            },
            _ => Err(TreeError::Container(
                ContainerErr::BadOperationOn(c_type,
                                             "active_border_color".into())))
        }
    }

    /// Sets the new output that the border should render on.
    pub fn update_border_output(&mut self, output: WlcOutput)
                                -> Result<(), ContainerErr> {
        let c_type = self.get_type();
        match *self {
            Container::View { handle, ref mut borders, .. }  => {
                handle.set_output(output);
                if let Some(borders_) = borders.take() {
                    *borders = borders_.set_output(output);
                }
                Ok(())
            }
            Container::Container { ref mut borders, .. } => {
                if let Some(borders_) = borders.take() {
                    *borders = borders_.set_output(output);
                }
                Ok(())
            },
            _ => {
                Err(ContainerErr::BadOperationOn(c_type,
                                                 "Can only call \
                                                  update_border_update on \
                                                  View/Container".into()))?
            }
        }
    }

    /// Gets the title for a wlc handle.
    /// Tries to get the title, then defers to class if blank,
    /// and finally to the app_id if that is blank as well.
    pub fn get_title(view: WlcView) -> String {
        let title = view.get_title();
        let class = view.get_class();
        if !title.is_empty() {
            title
        } else if !class.is_empty() {
            class
        } else {
            view.get_app_id()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_have_child() {
        let root = ContainerType::Root;
        let output = ContainerType::Output;
        let workspace = ContainerType::Workspace;
        let container = ContainerType::Container;
        let view = ContainerType::View;

        assert!(root.can_have_child(output),         "Root      > output");
        assert!(output.can_have_child(workspace),    "Output    > workspace");
        assert!(workspace.can_have_child(container), "Workspace > container");
        assert!(container.can_have_child(container), "Container > container");
        assert!(container.can_have_child(view),      "Container > view");

        assert!(!root.can_have_child(root),      "! Root > root");
        assert!(!root.can_have_child(workspace), "! Root > workspace");
        assert!(!root.can_have_child(container), "! Root > container");
        assert!(!root.can_have_child(view),      "! Root > view");

        assert!(!output.can_have_child(root),      "! Output > root");
        assert!(!output.can_have_child(output),    "! Output > output");
        assert!(!output.can_have_child(container), "! Output > container");
        assert!(!output.can_have_child(view),      "! Output > view");

        assert!(!workspace.can_have_child(root),      "! Workspace > root");
        assert!(!workspace.can_have_child(output),    "! Workspace > output");
        assert!(!workspace.can_have_child(workspace), "! Workspace > workspace");
        assert!(!workspace.can_have_child(view),      "! Workspace > view");

        assert!(!container.can_have_child(root),      "! Container > root");
        assert!(!container.can_have_child(workspace), "! Container > workspace");
        assert!(!container.can_have_child(output),    "! Container > container");

        assert!(!view.can_have_child(root),      "! View > root");
        assert!(!view.can_have_child(output),    "! View > output");
        assert!(!view.can_have_child(workspace), "! View > workspace");
        assert!(!view.can_have_child(container), "! View > container");
        assert!(!view.can_have_child(view),      "! View > view");
    }

    #[test]
    #[allow(unused_variables)]
    /// Tests set and get geometry
    fn geometry_test() {
        use rustwlc::*;
        let test_geometry1 = Geometry {
            origin: Point { x: 800, y: 600 },
            size: Size { w: 500, h: 500}
        };
        let test_geometry2 = Geometry {
            origin: Point { x: 1024, y: 2048},
            size: Size { w: 500, h: 700}
        };
        let root = Container::new_root();
        assert!(root.get_geometry().is_none());

        let output = Container::new_output(WlcView::root().as_output());

        let workspace = Container::new_workspace("1".to_string(),
                                                 Geometry {
                                                     origin: Point { x: 0, y: 0},
                                                     size: Size { w: 500, h: 500 }
                                                 });
        assert_eq!(workspace.get_geometry(), Some(Geometry {
            size: Size { w: 500, h: 500},
            origin: Point { x: 0, y: 0}
        }));
    }

    #[test]
    fn layout_change_test() {
        let root = Container::new_root();
        let output = Container::new_output(WlcView::root().as_output());
        let workspace = Container::new_workspace("1".to_string(),
                                                 Geometry {
                                                     origin: Point { x: 0, y: 0},
                                                     size: Size { w: 500, h: 500 }
                                                 });
        let mut container = Container::new_container(Geometry {
            origin: Point { x: 0, y: 0},
            size: Size { w: 0, h:0}
        },
                                                     WlcView::root().as_output(),
                                                     None);
        let view = Container::new_view(WlcView::root(), None);

        /* Container first, the only thing we can set the layout on */
        let layout = match container {
            Container::Container { ref layout, .. } => layout.clone(),
            _ => panic!()
        };
        assert_eq!(layout, Layout::Horizontal);
        let layouts = [Layout::Vertical, Layout::Horizontal];
        for new_layout in &layouts {
            container.set_layout(*new_layout).ok();
            let layout = match container {
                Container::Container { ref layout, .. } => layout.clone(),
                _ => panic!()
            };
            assert_eq!(layout, *new_layout);
        }

        for new_layout in &layouts {
            for container in &mut [root.clone(), output.clone(),
                                   workspace.clone(), view.clone()] {
                let result = container.set_layout(*new_layout);
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn floating_tests() {
        let mut root = Container::new_root();
        let mut output = Container::new_output(WlcView::root().as_output());
        let mut workspace = Container::new_workspace("1".to_string(),
                                                     Geometry {
                                                         origin: Point { x: 0, y: 0},
                                                         size: Size { w: 500, h: 500 }
                                                     });
        let mut container = Container::new_container(Geometry {
            origin: Point { x: 0, y: 0},
            size: Size { w: 0, h:0}
        }, WlcView::root().as_output(), None);
        let mut view = Container::new_view(WlcView::root(), None);
        // by default, none are floating.
        assert!(!root.floating());
        assert!(!output.floating());
        assert!(!workspace.floating());
        assert!(!container.floating());
        assert!(!view.floating());

        // trying to do anything to root, output, or workspace is Err.
        assert_eq!(root.set_floating(true),  Err(ContainerType::Root));
        assert_eq!(root.set_floating(false), Err(ContainerType::Root));
        assert_eq!(output.set_floating(true),  Err(ContainerType::Output));
        assert_eq!(output.set_floating(false), Err(ContainerType::Output));
        assert_eq!(workspace.set_floating(true),  Err(ContainerType::Workspace));
        assert_eq!(workspace.set_floating(false), Err(ContainerType::Workspace));

        assert_eq!(container.set_floating(true),  Ok(ContainerType::Container));
        assert!(container.floating());
        assert_eq!(container.set_floating(false), Ok(ContainerType::Container));
        assert!(!container.floating());

        assert_eq!(view.set_floating(true),  Ok(ContainerType::View));
        assert!(view.floating());
        assert_eq!(view.set_floating(false), Ok(ContainerType::View));
        assert!(!view.floating());
    }
}
