#![allow(unused_variables, dead_code)] // macros

/// Dbus macro for Layout code

use dbus;
use dbus_macros;

use uuid::Uuid;

use super::super::layout::try_lock_tree;
use super::super::layout::commands;

dbus_class!("org.way_cooler.Layout", class Layout {
    fn remove_active(&this) {
        commands::remove_active();
    }

    fn tile_switch(&this) {
        commands::tile_switch();
    }

    fn split_vertical(&this) {
        commands::split_vertical();
    }

    fn split_horizontal(&this) {
        commands::split_horizontal();
    }

    fn move_focus(&this, dir: &str) {
        match &*dir.to_lowercase() {
            "up" => commands::focus_up(),
            "down" => commands::focus_down(),
            "left" => commands::focus_left(),
            "right" => commands::focus_right(),
            _ => ()
        }
    }

    fn switch_to_workspace(&this, name: &str) {
        
    }

    fn focus_on(&this, id: &str) {
        
    }
});
