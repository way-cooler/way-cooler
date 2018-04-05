use wlroots::{self, Area, Origin, Size, project_box, WL_SHM_FORMAT_ARGB8888, Compositor, OutputHandler};

pub struct Output;

impl OutputHandler for Output {
    fn on_frame(&mut self, compositor: &mut Compositor, output: &mut wlroots::Output) {
        let renderer = compositor.renderer
            .as_mut()
            .expect("gles2 disabled");
        let mut renderer = renderer.render(output, None);
        renderer.clear([0.25, 0.25, 0.25, 1.0]);

    }
}
