use wlroots::{wlroots_dehandle, CompositorHandle, OutputBuilder, OutputBuilderResult, OutputManagerHandler};

pub struct OutputManager;

impl OutputManager {
    pub fn new() -> Self {
        OutputManager
    }
}

impl OutputManagerHandler for OutputManager {
    #[wlroots_dehandle(compositor, layout, cursor, output)]
    fn output_added<'output>(
        &mut self,
        compositor_handle: CompositorHandle,
        builder: OutputBuilder<'output>
    ) -> Option<OutputBuilderResult<'output>> {
        use compositor_handle as compositor;
        let server: &mut ::Server = compositor.into();
        let res = builder.build_best_mode(::Output);
        server.outputs.push(res.output.clone());
        let ::Server {
            ref mut cursor_handle,
            ref mut layout_handle,
            ref mut xcursor_manager,
            ..
        } = *server;
        use cursor_handle as cursor;
        use layout_handle as layout;
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
