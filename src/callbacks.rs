//! Implementations of the callbacks exposed by wlc.
//! These functions are the main entry points into Way Cooler from user action.
use rustwlc::handle::{WlcOutput, WlcView};
use rustwlc::types::*;

use super::keys;
use super::layout::{lock_tree, try_lock_tree, TreeError};
use super::lua;

use ::modes::{read_current_mode, EVENT_PASS_THROUGH};


pub extern fn output_created(output: WlcOutput) -> bool {
    trace!("output_created: {:?}: {}", output, output.get_name());
    if let Ok(mode) = read_current_mode() {
        mode.output_created(output)
    } else {
        false
    }
}

pub extern fn output_destroyed(output: WlcOutput) {
    trace!("output_destroyed: {:?}", output);
    if let Ok(mode) = read_current_mode() {
        mode.output_destroyed(output)
    }
}

pub extern fn output_focus(output: WlcOutput, focused: bool) {
    trace!("output_focus: {:?} focus={}", output, focused);
    if let Ok(mode) = read_current_mode() {
        mode.output_focused(output, focused)
    }
}

pub extern fn output_resolution(output: WlcOutput,
                                old_size_ptr: &Size, new_size_ptr: &Size) {
    trace!("output_resolution: {:?} from  {:?} to {:?}",
           output, *old_size_ptr, *new_size_ptr);
    if let Ok(mode) = read_current_mode() {
        mode.output_resolution(output, *old_size_ptr, *new_size_ptr)
    }
}

pub extern fn post_render(output: WlcOutput) {
    if let Ok(mode) = read_current_mode() {
        mode.output_render_post(output)
    }
}

pub extern fn view_created(view: WlcView) -> bool {
    trace!("view_created: {:?}: \"{}\"", view, view.get_title());
    if let Ok(mode) = read_current_mode() {
        mode.view_created(view)
    } else {
        false
    }
}

pub extern fn view_destroyed(view: WlcView) {
    trace!("view_destroyed: {:?}", view);
    if let Ok(mode) = read_current_mode() {
        mode.view_destroyed(view)
    }
}

pub extern fn view_focus(current: WlcView, focused: bool) {
    if let Ok(lock_screen) = try_lock_lock_screen() {
        if let Some(ref lock_screen) = *lock_screen {
            lock_screen.view().map(|v| v.set_state(VIEW_ACTIVATED, focused));
            return
        }
    }
    trace!("view_focus: {:?} {}", current, focused);
    if let Ok(mode) = read_current_mode() {
        mode.view_focused(current, focused)
    }
}

pub extern fn view_props_changed(view: WlcView, prop: ViewPropertyType) {
    trace!("view_props_changed for view {:?}: {:?}", view, prop);
    if let Ok(mode) = read_current_mode() {
        mode.view_props_changed(view, prop)
    }
}

pub extern fn view_move_to_output(current: WlcView,
                                  o1: WlcOutput, o2: WlcOutput) {
    trace!("view_move_to_output: {:?}, {:?}, {:?}", current, o1, o2);
    if let Ok(mode) = read_current_mode() {
        mode.view_moved_to_output(current, o1, o2)
    }
}

pub extern fn view_request_state(view: WlcView, state: ViewState, toggle: bool) {
    if let Ok(lock_screen) = lock_lock_screen() {
        if lock_screen.is_some() {
            return
        }
    }
    trace!("Setting {:?} to state {:?}", view, state);
    if let Ok(mode) = read_current_mode() {
        mode.view_request_state(view, state, toggle)
    }
}

pub extern fn view_request_move(view: WlcView, dest: &Point) {
    trace!("View {:?} request to move to {:?}", view, dest);
    if let Ok(mode) = read_current_mode() {
        mode.view_request_move(view, *dest)
    }
}

pub extern fn view_request_resize(view: WlcView, edge: ResizeEdge, point: &Point) {
    trace!("View {:?} request resize w/ edge {:?} to point {:?}",
           view, edge, point);
    if let Ok(mode) = read_current_mode() {
        mode.view_request_resize(view, edge, *point)
    }
}

pub extern fn view_request_geometry(view: WlcView, geometry: &Geometry) {
    trace!("View {:?} requested geometry {:?}", view, geometry);
    if let Ok(mode) = read_current_mode() {
        mode.view_request_geometry(view, *geometry)
    }
}

pub extern fn keyboard_key(view: WlcView, time: u32, mods: &KeyboardModifiers,
                           key: u32, state: KeyState) -> bool {
    // NOTE We read the mode out here, so that if there is a keybinding
    // that changes the mode we can do that without being blocked!
    let mode = if let Ok(mode) = read_current_mode() {
        *mode
    } else {
        return EVENT_PASS_THROUGH
    };
    mode.on_keyboard_key(view, time, *mods, key, state)
}

pub extern fn pointer_button(view: WlcView, time: u32,
                         mods: &KeyboardModifiers, button: u32,
                             state: ButtonState, point: &Point) -> bool {
    if let Ok(mode) = read_current_mode() {
        mode.on_pointer_button(view, time, *mods, button, state, *point)
    } else {
        EVENT_PASS_THROUGH
    }
}

pub extern fn pointer_scroll(view: WlcView, time: u32,
                         mods_ptr: &KeyboardModifiers, axis: ScrollAxis,
                         heights: [f64; 2]) -> bool {
    if let Ok(mode) = read_current_mode() {
        mode.on_pointer_scroll(view, time, *mods_ptr, axis, heights)
    } else {
        EVENT_PASS_THROUGH
    }
}

pub extern fn pointer_motion(view: WlcView, time: u32, point: &Point) -> bool {
    if let Ok(mode) = read_current_mode() {
        mode.on_pointer_motion(view, time, *point)
    } else {
        EVENT_PASS_THROUGH
    }
}

pub extern fn compositor_ready() {
    info!("Preparing compositor!");
    info!("Initializing Lua...");
    lua::init();
    keys::init();
}

pub extern fn compositor_terminating() {
    info!("Compositor terminating!");
    lua::send(lua::LuaQuery::Terminate).ok();
    if let Ok(mut tree) = try_lock_tree() {
        if tree.destroy_tree().is_err() {
            error!("Could not destroy tree");
        }
    }

}

pub extern fn view_pre_render(view: WlcView) {
    if let Ok(mut tree) = lock_tree() {
        tree.render_borders(view).unwrap_or_else(|err| {
            match err {
                // TODO Only filter if background or bar
                TreeError::ViewNotFound(_) => {},
                err => warn!("Error while rendering borders: {:?}", err)
            }
        })
    }
}


pub fn init() {
    use rustwlc::callback;

    callback::output_created(output_created);
    callback::output_destroyed(output_destroyed);
    callback::output_focus(output_focus);
    callback::output_resolution(output_resolution);
    callback::output_render_post(post_render);
    callback::view_created(view_created);
    callback::view_destroyed(view_destroyed);
    callback::view_focus(view_focus);
    callback::view_move_to_output(view_move_to_output);
    callback::view_request_geometry(view_request_geometry);
    callback::view_request_state(view_request_state);
    callback::view_request_move(view_request_move);
    callback::view_request_resize(view_request_resize);
    callback::view_properties_changed(view_props_changed);
    callback::keyboard_key(keyboard_key);
    callback::pointer_button(pointer_button);
    callback::pointer_scroll(pointer_scroll);
    callback::pointer_motion(pointer_motion);
    callback::compositor_ready(compositor_ready);
    callback::compositor_terminate(compositor_terminating);
    callback::view_render_pre(view_pre_render);
    trace!("Registered wlc callbacks");
}
