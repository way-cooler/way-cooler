use compositor::{Server, Shell, View};
use wlroots::{CompositorHandle, XdgV6ShellHandler, XdgV6ShellManagerHandler, XdgV6ShellState::*,
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
                   compositor: CompositorHandle,
                   mut shell_surface: XdgV6ShellSurfaceHandle)
                   -> Option<Box<XdgV6ShellHandler>> {
        let is_toplevel = with_handles!([(shell_surface: {&mut shell_surface})] => {
            match shell_surface.state().unwrap() {
                TopLevel(_) => true,
                _ => false
            }
        }).unwrap();

        if is_toplevel {
            with_handles!([(compositor: {compositor})] => {
                let server: &mut Server = compositor.into();
                server.views
                    .push(View::new(Shell::XdgV6(shell_surface.into())));
            }).unwrap();
        }

        Some(Box::new(XdgV6::new()))
    }

    fn surface_destroyed(&mut self,
                         compositor: CompositorHandle,
                         shell_surface: XdgV6ShellSurfaceHandle) {
        with_handles!([(compositor: {compositor})] => {
            let server: &mut Server = compositor.into();
            let destroyed_shell = shell_surface.into();
            if let Some(pos) = server.views
                .iter()
                .position(|view| view.shell == destroyed_shell)
            {
                server.views.remove(pos);
            }
        }).unwrap();
    }
}
