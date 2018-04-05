use compositor::{self, Server, Shell};
use std::time::{SystemTime, UNIX_EPOCH};
use wlroots::{self, project_box, Area, Compositor, Origin, OutputHandler, Size,
              WL_SHM_FORMAT_ARGB8888};

pub struct Output;

impl OutputHandler for Output {
    fn on_frame(&mut self, compositor: &mut Compositor, output: &mut wlroots::Output) {
        let state: &mut Server = compositor.data.downcast_mut().unwrap();
        let Server { ref mut layout,
                     ref mut views,
                     .. } = *state;
        let renderer = compositor.renderer.as_mut().expect("gles2 disabled");
        let mut renderer = renderer.render(output, None);
        renderer.clear([0.25, 0.25, 0.25, 1.0]);
        for view in views {
            let mut surface = view.shell.surface();
            run_handles!([(surface: {surface}),
                          (layout: {&mut *layout})] => {
                let (width, height) = surface.current_state().size();
                let (render_width, render_height) =
                    (width * renderer.output.scale() as i32,
                     height * renderer.output.scale() as i32);
                let (lx, ly) = (0.0, 0.0);
                let render_box = Area::new(Origin::new(lx as i32, ly as i32),
                                           Size::new(render_width,
                                                     render_height));
                if layout.intersects(renderer.output, render_box) {
                    let transform = renderer.output.get_transform().invert();
                    let matrix = project_box(render_box,
                                             transform,
                                             0.0,
                                             renderer.output
                                             .transform_matrix());
                    renderer.render_texture_with_matrix(&surface.texture(),
                                                        matrix);
                    let start = SystemTime::now();
                    let now = start.duration_since(UNIX_EPOCH)
                        .expect("Time went backwards");
                    surface.send_frame_done(now);
                }
            })
                .expect("Surface was destroyed")
                .expect("Layout was destroyed")
        }
    }
}
