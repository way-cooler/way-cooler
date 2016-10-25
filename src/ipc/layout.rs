#![allow(unused_variables, dead_code)] // macros

/// Dbus macro for Layout code

use super::{DBusTree, DBusResult};
use super::utils;

use dbus::tree::MethodErr;

use uuid::Uuid;

use super::super::layout::try_lock_tree;
use super::super::layout::Direction;
use super::super::layout::commands as layout_cmd;

dbus_interface! {
    path: "/org/way_cooler/Layout";
    name: "org.way_cooler.Layout";

    fn ToggleFloat(container_id: String) -> success: DBusResult<bool> {
        if container_id == "" {
            layout_cmd::toggle_float();
            Ok(true)
        } else {
            let uuid = try!(Uuid::parse_str(&container_id).map_err(
                |uuid_err| MethodErr::invalid_arg(&"uuid is not valid")));
            if let Ok(mut tree) = try_lock_tree() {
                match tree.toggle_float() {
                    Ok(_) => Ok(true),
                    Err(err) => return Err(MethodErr::failed(&"could not toggle float"))
                }
            } else {
                return Err(MethodErr::failed(&"could not lock tree"))
            }
        }
    }

    fn MoveContainer(container_id: String, direction: String) -> success: DBusResult<bool> {
        let target_uuid = match &*container_id {
            "" => None,
            uuid => Some(try!(Uuid::parse_str(uuid).map_err(
                |uuid_err| MethodErr::invalid_arg(&"uuid is not valid"))))
        };
        let direction = try!(match &*direction.to_lowercase() {
            "up" => Ok(Direction::Up),
            "down" => Ok(Direction::Down),
            "left" => Ok(Direction::Left),
            "right" => Ok(Direction::Right),
            other => Err(MethodErr::invalid_arg(&"direction is not a valid direction"))
        });
        if let Ok(mut tree) = try_lock_tree() {
            match tree.move_active(target_uuid, direction) {
                Ok(_) => Ok(true),
                Err(err) => Err(MethodErr::failed(&"could not move that container"))
            }
        } else {
            Err(MethodErr::failed(&"could not lock tree"))
        }
    }

    fn ActiveContainerId() -> container_id: DBusResult<String> {
        if let Ok(mut tree) = try_lock_tree() {
            match tree.active_id() {
                Some(id) => Ok(id.to_string()),
                None => Ok("".to_string())
            }
        } else {
            Err(MethodErr::failed(&"could not read from the tree"))
        }
    }

    fn SplitContainer(container_id: String, split_direction: String) -> success: DBusResult<bool> {
        let uuid = try!(utils::parse_uuid("container_id", &container_id));
        let direction = try!(utils::parse_direction("split_direction", &split_direction));

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
