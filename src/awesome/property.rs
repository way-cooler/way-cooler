use super::class::PropF;

pub struct Property {
    name: String,
    cb_new: Option<PropF>,
    cb_index: Option<PropF>,
    cb_newindex: Option<PropF>
}

impl Property {
    pub fn new(name: String,
               cb_new: Option<PropF>,
               cb_index: Option<PropF>,
               cb_newindex: Option<PropF>) -> Self {
        Property { name, cb_new, cb_index, cb_newindex }
    }
}
