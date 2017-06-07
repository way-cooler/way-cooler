use super::util::*;

#[test]
fn ensure_workspace_switch_when_fullscreen() {
    let mut tree = basic_tree();
    let active_id = tree.active_id().unwrap();
    tree.set_fullscreen(active_id, true).unwrap();
    tree = tree.add_new_view().unwrap();
    tree = tree.add_workspace("2").unwrap();
    tree = tree.add_workspace("1").unwrap();
    assert_eq!(tree.active_id(), Some(active_id));
}
