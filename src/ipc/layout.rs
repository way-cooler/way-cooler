#![allow(unused_variables, dead_code)] // macros

/// Dbus macro for Layout code

use super::utils::{parse_uuid, parse_direction};

use dbus::tree::MethodErr;

use super::super::layout::try_lock_tree;
use super::super::layout::Direction;
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
        let direction = match direction.to_lowercase().as_str() {
            "up" => Direction::Up,
            "down" => Direction::Down,
            "left" => Direction::Left,
            "right" => Direction::Right,
            other => return Err(MethodErr::invalid_arg(&"Not a valid direction"))
        };
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

    fn SplitContainer(container_id: String, split_direction: String) -> success: DBusResult<bool> {
        let uuid = try!(parse_uuid("container_id", &container_id));
        let direction = try!(parse_direction("split_direction", &split_direction));

        if let Ok(mut tree) = try_lock_tree() {
            match tree.move_active(uuid, direction) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false)
            }
        } else {
            Err(MethodErr::failed(&"Could not lock tree"))
        }
    }
}
