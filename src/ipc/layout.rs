#![allow(unused_variables, dead_code)] // macros

/// Dbus macro for Layout code

use super::utils::{parse_uuid, parse_direction, parse_axis};

use dbus::tree::MethodErr;

use super::super::layout::try_lock_tree;
use super::super::layout::{Layout};
use super::super::layout::commands as layout_cmd;

dbus_interface! {
    path: "/org/way_cooler/Layout";
    name: "org.way_cooler.Layout";

    fn ToggleFloat(container_id: String) -> success: DBusResult<bool> {
        let maybe_uuid = try!(parse_uuid("container_id", &container_id));
        match maybe_uuid {
            Some(uuid) => {
                match try_lock_tree() {
                    Ok(mut tree) => {
                        tree.toggle_float()
                            .and(Ok(true))
                            .map_err(|err| {
                                MethodErr::failed(&format!("{:?}", err))
                            })
                    },
                    Err(err) => Err(MethodErr::failed(&format!("{:?}", err)))
                }
            },
            None => {
                layout_cmd::toggle_float();
                Ok(true)
            }
        }
    }

    fn MoveContainer(container_id: String, direction: String) -> success: DBusResult<bool> {
        let target_uuid = try!(parse_uuid("container_id", &container_id));
        let direction = try!(parse_direction("direction", direction.as_str()));
        match try_lock_tree() {
            Ok(mut tree) => {
                match tree.move_active(target_uuid, direction) {
                    Ok(_) => Ok(true),
                    Err(err) => Err(MethodErr::failed(&format!("{:?}", err)))
                }
            },
            Err(err) => Err(MethodErr::failed(&format!("{:?}", err)))
        }
    }

    fn ActiveContainerId() -> container_id: DBusResult<String> {
        match try_lock_tree() {
            Ok(tree) => {
                match tree.active_id() {
                    Some(id) => Ok(id.to_string()),
                    None => Ok("".to_string())
                }
            },
            Err(err) => Err(MethodErr::failed(&format!("{:?}", err)))
        }
    }

    fn SplitContainer(container_id: String, split_axis: String) -> success: DBusResult<bool> {
        let uuid = try!(parse_uuid("container_id", &container_id));
        let axis = try!(parse_axis("split_direction", split_axis.as_str()));
        // TODO Tree commands need to have these defined on the Tree,
        // for now this is _ok_, but we are swallowing an potential Tree lock error here.
        match axis {
            Layout::Horizontal => layout_cmd::split_horizontal(),
            Layout::Vertical => layout_cmd::split_vertical()
        }
            Ok(true)
    }
}
