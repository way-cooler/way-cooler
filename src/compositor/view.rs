use compositor::Shell;
use wlroots::Origin;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct View {
    pub shell: Shell,
    pub origin: Origin
}

impl View {
    pub fn new(shell: Shell) -> View {
        View { shell,
               origin: Origin::default() }
    }
}
