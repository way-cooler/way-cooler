use std::rc::Rc;

use wlroots::{
    project_box, utils::current_time, wlroots_dehandle::wlroots_dehandle, Area, CompositorHandle, Origin,
    OutputHandle, OutputHandler, OutputLayoutHandle, Renderer, Size, SurfaceHandle
};

use Server;

pub struct Output;

impl OutputHandler for Output {
    #[wlroots_dehandle(compositor, output)]
    fn on_frame(&mut self, compositor_handle: CompositorHandle, output_handle: OutputHandle) {
        use compositor_handle as compositor;
        use output_handle as output;
        let state: &mut Server = compositor.data.downcast_mut().unwrap();
        let Server {
            ref mut layout_handle,
            ref mut views,
            ..
        } = *state;
        let renderer = compositor.renderer.as_mut().expect("gles2 disabled");
        let mut renderer = renderer.render(output, None);
        renderer.clear([0.25, 0.25, 0.25, 1.0]);
        render_views(&mut renderer, layout_handle, views);
    }
}

#[allow(dead_code)]
#[wlroots_dehandle(surface, layout)]
fn render_surface(
    renderer: &mut Renderer,
    layout_handle: &mut OutputLayoutHandle,
    surface_handle: &mut SurfaceHandle,
    lx: i32,
    ly: i32
) {
    use layout_handle as layout;
    use surface_handle as surface;
    let (width, height) = surface.current_state().size();
    let (render_width, render_height) = (
        width * renderer.output.scale() as i32,
        height * renderer.output.scale() as i32
    );
    let render_box = Area::new(Origin::new(lx, ly), Size::new(render_width, render_height));

    if layout.intersects(renderer.output, render_box) {
        let transform = renderer.output.get_transform().invert();
        let matrix = project_box(render_box, transform, 0.0, renderer.output.transform_matrix());
        if !renderer.render_texture_with_matrix(surface.texture().as_ref().unwrap(), matrix) {
            warn!("Could not render a surface");
        }
        surface.send_frame_done(current_time());
    }
}

/// Render all of the client views.
#[wlroots_dehandle(surface, layout)]
fn render_views(
    renderer: &mut Renderer,
    layout_handle: &mut OutputLayoutHandle,
    views: &mut Vec<Rc<::View>>
) {
    for view in views.iter_mut().rev() {
        let origin = view.origin.get();
        view.for_each_surface(&mut |surface_handle: SurfaceHandle, sx, sy| {
            use layout_handle as layout;
            use surface_handle as surface;
            let (width, height) = surface.current_state().size();
            let (render_width, render_height) = (
                width * renderer.output.scale() as i32,
                height * renderer.output.scale() as i32
            );
            let render_box = Area::new(
                Origin::new(origin.x + sx, origin.y + sy),
                Size::new(render_width, render_height)
            );

            if layout.intersects(renderer.output, render_box) {
                let transform = renderer.output.get_transform().invert();
                let matrix = project_box(render_box, transform, 0.0, renderer.output.transform_matrix());
                if !renderer.render_texture_with_matrix(surface.texture().as_ref().unwrap(), matrix) {
                    warn!("Could not render a surface");
                }
                surface.send_frame_done(current_time());
            }
        });
    }
}
