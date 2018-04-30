use std::time::{SystemTime, UNIX_EPOCH};

use cairo::Context;
use wlroots::{self, project_box, Area, Compositor, OutputHandler, Size, WL_SHM_FORMAT_ARGB8888};

use compositor::Server;

pub struct Output;

impl OutputHandler for Output {
    fn on_frame(&mut self, compositor: &mut Compositor, output: &mut wlroots::Output) {
        let state: &mut Server = compositor.data.downcast_mut().unwrap();
        let Server { ref mut layout,
                     ref mut views,
                     ref mut drawins,
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
                let render_box = Area::new(view.origin,
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
            }).expect("Could not render views")
        }
        error!("drawins: {:?}", drawins.len());
        for &(ref drawin, geometry) in drawins.iter() {
            // NOTE No need to check if it's visible since if it's in the list
            // then it's implicitly visible to the user
            let mut lock = drawin.image();
            let mut image = match lock {
                Err(_) => continue,
                Ok(ref none) if none.is_none() => continue,
                Ok(ref mut image) => image.as_mut().unwrap()
            };
            {
                let cr = Context::new(&*image);
                cr.set_source_surface(&*image, 0.0, 0.0);
                cr.paint();
            }
            let Area { size: Size { width, height },
                       .. } = geometry;
            let data = &mut *image.get_data().expect("Non-unique lock on image");
            let texture = renderer.create_texture_from_pixels(WL_SHM_FORMAT_ARGB8888,
                                                              (width * 4) as _,
                                                              width as _,
                                                              height as _,
                                                              data)
                                  .expect("Could not allocate texture");
            let transform_matrix = renderer.output.transform_matrix();
            let inverted_transform = renderer.output.get_transform().invert();
            let matrix = project_box(geometry, inverted_transform, 0.0, transform_matrix);
            renderer.render_texture_with_matrix(&texture, matrix);
        }
    }
}
