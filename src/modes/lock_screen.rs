//! Implementations of the default callbacks exposed by wlc.
//! These functions are the main entry points into Way Cooler from user action.
//! This is the default mode that Way Cooler is in at initilization
use std::iter;
use rustwlc::*;
use rustwlc::input::{pointer, keyboard};
use rustwlc::render::{read_pixels, wlc_pixel_format};
use uuid::Uuid;

use super::{EVENT_BLOCKED, EVENT_PASS_THROUGH, LEFT_CLICK, RIGHT_CLICK};
use ::keys::{self, KeyPress, KeyEvent};
use ::layout::{lock_tree, try_lock_tree, try_lock_action, Action, ContainerType,
               MovementError, TreeError, FocusError};
use ::layout::commands::set_performing_action;
use ::lua::{self, LuaQuery};

use ::render::screen_scrape::{read_screen_scrape_lock, scraped_pixels_lock,
                              sync_scrape};

use registry::{self};
use super::default::Default;
use super::Mode;


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LockScreen(::lockscreen::LockScreen);

#[allow(unused)]
impl Mode for LockScreen {
    fn output_created(&self, output: WlcOutput) -> bool {
        Default.output_created(output)
    }

    fn output_destroyed(&self, output: WlcOutput) {
        Default.output_destroyed(output)
    }

    fn output_focused(&self, output: WlcOutput, focused: bool) {
        Default.output_focused(output, focused)
    }

    fn output_resolution(&self, output: WlcOutput,
                                    old_size_ptr: Size, new_size_ptr: Size) {
        Default.output_resolution(output, old_size_ptr, new_size_ptr)
    }

    fn output_render_post(&self, output: WlcOutput) {
        // Don't allow screen scraping during lock screen
        let need_to_fetch = read_screen_scrape_lock();
        if *need_to_fetch {
            if let Ok(mut scraped_pixels) = scraped_pixels_lock() {
                let resolution = output.get_resolution()
                    .expect("Output had no resolution");
                // Give them zero-ed out pixels
                let size = (resolution.w * resolution.h * 4) as usize;
                *scraped_pixels = iter::repeat(0).take(size).collect();
                sync_scrape();
            }
        }
    }

    fn view_created(&self, view: WlcView) -> bool {
        // this will focus and set the size if necessary.
        if self.0.add_view_if_match(view) {
            trace!("Adding lockscreen");
            true
        } else {
            false
        }
    }

    fn view_destroyed(&self, view: WlcView) {
        if self.0.remove_if_match(view).is_none() {
            trace!("Removing lock screen");
            // TODO CHANGE MODE
            return
        }
    }

    fn view_focused(&self, current: WlcView, focused: bool) {
        self.0.view().map(|v| v.set_state(VIEW_ACTIVATED, focused));
    }

    fn view_props_changed(&self, view: WlcView, prop: ViewPropertyType) {
        Default.view_props_changed(view, prop)
    }

    fn view_moved_to_output(&self, view: WlcView, o1: WlcOutput, o2: WlcOutput) {
        Default.view_moved_to_output(view, o1, o2)
    }

    fn view_request_state(&self, view: WlcView, state: ViewState, toggle: bool) {
        // Do nothing
    }

    fn view_request_move(&self, view: WlcView, _dest: Point) {
        // Do nothing
    }

    fn view_request_resize(&self, view: WlcView, edge: ResizeEdge, point: Point) {
        // Do nothing
    }

    fn on_keyboard_key(&self, _view: WlcView, _time: u32, mods: KeyboardModifiers,
                            key: u32, state: KeyState) -> bool {
        self.0.view().map(WlcView::focus);
        EVENT_PASS_THROUGH
    }

    fn view_request_geometry(&self, view: WlcView, geometry: Geometry) {
        // Do nothing
    }

    fn on_pointer_button(&self, view: WlcView, _time: u32,
                            mods: KeyboardModifiers, button: u32,
                                state: ButtonState, point: Point) -> bool {
        self.0.view().map(WlcView::focus);
        EVENT_PASS_THROUGH
    }

    fn on_pointer_scroll(&self, _view: WlcView, _time: u32,
                            _mods_ptr: KeyboardModifiers, _axis: ScrollAxis,
                            _heights: [f64; 2]) -> bool {
        EVENT_PASS_THROUGH
    }

    fn on_pointer_motion(&self, view: WlcView, _time: u32, point: Point) -> bool {
        pointer::set_position(point);
        EVENT_PASS_THROUGH
    }

    fn view_pre_render(&self, view: WlcView) {
        Default.view_pre_render(view)
    }
}
