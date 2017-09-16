use super::class::PropF;

pub struct Property {
    name: String,
    cb_new: PropF,
    cb_index: PropF,
    cb_newindex: PropF
}
