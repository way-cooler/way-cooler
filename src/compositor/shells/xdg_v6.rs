use compositor::{self, Server, Shell, View};
use wlroots::{Compositor, XdgV6ShellHandler, XdgV6ShellManagerHandler, XdgV6ShellSurface,
              XdgV6ShellSurfaceHandle};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct XdgV6 {
    shell_surface: XdgV6ShellSurfaceHandle
}

impl XdgV6 {
    pub fn new() -> Self {
        XdgV6 { ..XdgV6::default() }
    }
}

impl XdgV6ShellHandler for XdgV6 {}

pub struct XdgV6ShellManager;

impl XdgV6ShellManagerHandler for XdgV6ShellManager {
    fn new_surface(&mut self,
                   compositor: &mut Compositor,
                   shell_surface: &mut XdgV6ShellSurface)
                   -> Option<Box<XdgV6ShellHandler>> {
        let server: &mut Server = compositor.into();
        server.views
              .push(View::new(Shell::XdgV6(shell_surface.weak_reference().into())));
        Some(Box::new(XdgV6::new()))
    }

    fn surface_destroyed(&mut self,
                         compositor: &mut Compositor,
                         shell_surface: &mut XdgV6ShellSurface) {
        let server: &mut Server = compositor.into();
        let destroyed_shell = shell_surface.weak_reference().into();
        if let Some(pos) = server.views
                                 .iter()
                                 .position(|view| view.shell == destroyed_shell)
        {
            server.views.remove(pos);
        }
    }
}
