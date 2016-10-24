#![allow(unused_variables, dead_code)] // macros

/// Dbus macro for Layout code

use super::{DBusTree, DBusResult};

use dbus::tree::MethodErr;

use uuid::Uuid;

use super::super::layout::try_lock_tree;
use super::super::layout::commands as layout_cmd;

dbus_interface! {
    path: "/org/way_cooler/Layout";
    name: "org.way-cooler.Layout";
    fn ToggleFloat(uuid: String) -> success: DBusResult<()> {
        if uuid == "" {
            layout_cmd::toggle_float();
            Ok()
        } else {
            let uuid = try!(Uuid::parse_str(&uuid).map_err(
                |uuid_err| MethodErr::invalid_arg("uuid is not valid")));
            if let Ok(mut tree) = try_lock_tree() {
                match tree.toggle_float() {
                    Ok(_) => Ok(()),
                    Err(err) => return Err(MethodErr::failed(""))
                }
            }
        }
    }
}
