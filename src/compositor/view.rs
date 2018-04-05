use compositor::Shell;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct View {
    pub shell: Shell
}

impl View {
    pub fn new(shell: Shell) -> View {
        View { shell }
    }
}
