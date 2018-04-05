use compositor::{Output, Server};
use wlroots::{Compositor, OutputBuilder, OutputBuilderResult, OutputManagerHandler};

pub struct OutputManager;

impl OutputManager {
    pub fn new() -> Self {
        OutputManager
    }
}

impl OutputManagerHandler for OutputManager {
    fn output_added<'output>(&mut self,
                             compositor: &mut Compositor,
                             builder: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        let server: &mut Server = compositor.into();
        let res = builder.build_best_mode(Output);
        let Server { ref mut cursor,
                     ref mut layout,
                     ref mut xcursor_theme,
                     .. } = *server;
        run_handles!([(layout: {layout}), (cursor: {cursor})] => {
            let xcursor = xcursor_theme.get_cursor("left_ptr".into())
                .expect("Could not load left_ptr cursor");
            layout.add_auto(res.output);
            cursor.attach_output_layout(layout);
            cursor.set_cursor_image(&xcursor.images()[0]);
            let (x, y) = cursor.coords();
            cursor.warp(None, x, y);
        }).expect("Layout was destroyed").expect("Cursor was destroyed");
        Some(res)
    }
}
