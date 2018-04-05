use wlroots::{Compositor, OutputManagerHandler, OutputBuilder, OutputBuilderResult};
use compositor::Output;

pub struct OutputManager;

impl OutputManager {
    pub fn new() -> Self {
        OutputManager
    }
}

impl OutputManagerHandler for OutputManager {
    fn output_added<'output>(&mut self,
                             _: &mut Compositor,
                             builder: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        Some(builder.build_best_mode(Output))
    }
}
