use wlroots::{CompositorHandle, OutputBuilder, OutputBuilderResult, OutputManagerHandler};
use wlroots::wlroots_dehandle;

pub struct OutputManager;

impl OutputManager {
    pub fn new() -> Self {
        OutputManager
    }
}

impl OutputManagerHandler for OutputManager {
    #[wlroots_dehandle(compositor, layout, cursor, output)]
    fn output_added<'output>(&mut self,
                             compositor: CompositorHandle,
                             builder: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        use compositor as compositor;
        let server: &mut ::Server = compositor.into();
        let res = builder.build_best_mode(::Output);
        server.outputs.push(res.output.clone());
        let ::Server { ref mut cursor,
                       ref mut layout,
                       ref mut xcursor_manager,
                       .. } = *server;
        use layout as layout;
        use cursor as cursor;
        {
            let output_handle = &res.output;
            use output_handle as output;
            layout.add_auto(output);
            cursor.attach_output_layout(layout);
            xcursor_manager.load(output.scale());
            xcursor_manager.set_cursor_image("left_ptr".to_string(), cursor);
            let (x, y) = cursor.coords();
            cursor.warp(None, x, y);
        }
        Some(res)
    }
}
