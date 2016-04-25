//! Tests for containers.rs

#[cfg(test)]
mod tests {
    use rustwlc::handle::{WlcView, WlcOutput};
    use std::rc::*;
    use super::super::containers::{Container, ContainerType, Layout, Node};

    #[test]
    fn it_works() {
        let mut result = vec!();
        for _ in 1..10 {
            result.push(Container::new_root());
        }
        for thing in result {
            drop(thing);
        }
        //assert_eq!(result[
    }
}
