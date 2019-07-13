#include "cursor.h"

#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <wlr/util/log.h>

#include "mousegrabber.h"
#include "output.h"
#include "seat.h"
#include "server.h"
#include "view.h"

static void wc_process_motion(struct wc_server *server, uint32_t time) {
	struct wc_seat *seat = server->seat;
	struct wc_cursor *cursor = server->cursor;
	struct wlr_cursor *wlr_cursor = server->cursor->wlr_cursor;
	struct wc_view *view = cursor->grabbed.view;

	switch (cursor->cursor_mode) {
	case WC_CURSOR_MOVE: {
		wc_view_damage_whole(view);

		view->geo.x = wlr_cursor->x - cursor->grabbed.original_x;
		view->geo.y = wlr_cursor->y - cursor->grabbed.original_y;

		wc_view_damage_whole(view);
		break;
	}
	case WC_CURSOR_RESIZE: {
		int dx = wlr_cursor->x - cursor->grabbed.original_x;
		int dy = wlr_cursor->y - cursor->grabbed.original_y;
		struct wlr_box new_geo = {
				.x = view->geo.x,
				.y = view->geo.y,
				.width = cursor->grabbed.original_view_geo.width,
				.height = cursor->grabbed.original_view_geo.height,
		};

		if (cursor->grabbed.resize_edges & WLR_EDGE_TOP) {
			new_geo.y = cursor->grabbed.original_view_geo.y + dy;
			new_geo.height -= dy;
			if (new_geo.height < 1) {
				new_geo.y += new_geo.height;
			}
		} else if (cursor->grabbed.resize_edges & WLR_EDGE_BOTTOM) {
			new_geo.height += dy;
		}

		if (cursor->grabbed.resize_edges & WLR_EDGE_LEFT) {
			new_geo.x = cursor->grabbed.original_view_geo.x + dx;
			new_geo.width -= dx;
			if (new_geo.width < 1) {
				new_geo.x += new_geo.width;
			}
		} else if (cursor->grabbed.resize_edges & WLR_EDGE_RIGHT) {
			new_geo.width += dx;
		}

		wc_view_update_geometry(view, new_geo);

		break;
	}
	case WC_CURSOR_PASSTHROUGH: {
		double sx, sy;
		struct wlr_surface *surface = NULL;
		struct wc_view *view = wc_view_at(
				server, wlr_cursor->x, wlr_cursor->y, &sx, &sy, &surface);
		bool cursor_image_not_default =
				!cursor->image || strcmp(cursor->image, "left_ptr") != 0;

		if (!view && cursor_image_not_default) {
			cursor->image = "left_ptr";
			wlr_xcursor_manager_set_cursor_image(
					server->xcursor_mgr, "left_ptr", cursor->wlr_cursor);
		}

		wc_seat_update_surface_focus(seat, surface, sx, sy, time);
		break;
	}
	}

	struct wlr_output *active_output = wlr_output_layout_output_at(
			server->output_layout, wlr_cursor->x, wlr_cursor->y);
	if (active_output != server->active_output->wlr_output) {
		struct wc_output *output_;
		wl_list_for_each(output_, &server->outputs, link) {
			if (output_->wlr_output == active_output) {
				server->active_output = output_;
				break;
			}
		}
	}

	wc_mousegrabber_notify_mouse_moved(server, wlr_cursor->x, wlr_cursor->y);
}

static void wc_cursor_motion(struct wl_listener *listener, void *data) {
	struct wc_cursor *cursor = wl_container_of(listener, cursor, motion);
	struct wlr_event_pointer_motion *event = data;

	wlr_cursor_move(
			cursor->wlr_cursor, event->device, event->delta_x, event->delta_y);
	wc_process_motion(cursor->server, event->time_msec);
}

static void wc_cursor_motion_absolute(
		struct wl_listener *listener, void *data) {
	struct wc_cursor *cursor =
			wl_container_of(listener, cursor, motion_absolute);
	struct wlr_event_pointer_motion_absolute *event = data;

	wlr_cursor_warp_absolute(
			cursor->wlr_cursor, event->device, event->x, event->y);
	wc_process_motion(cursor->server, event->time_msec);
}

static void wc_cursor_button(struct wl_listener *listener, void *data) {
	struct wc_cursor *cursor = wl_container_of(listener, cursor, button);
	struct wc_server *server = cursor->server;
	struct wlr_event_pointer_button *event = data;
	wlr_seat_pointer_notify_button(
			server->seat->seat, event->time_msec, event->button, event->state);

	double sx, sy;
	struct wlr_surface *surface = NULL;
	struct wc_view *view = wc_view_at(server, cursor->wlr_cursor->x,
			cursor->wlr_cursor->y, &sx, &sy, &surface);
	if (event->state == WLR_BUTTON_RELEASED) {
		cursor->cursor_mode = WC_CURSOR_PASSTHROUGH;
	} else if (view != NULL) {
		wc_focus_view(view);
	}
}

static void wc_cursor_axis(struct wl_listener *listener, void *data) {
	struct wc_cursor *cursor = wl_container_of(listener, cursor, axis);
	struct wc_server *server = cursor->server;
	struct wlr_event_pointer_axis *event = data;

	wlr_seat_pointer_notify_axis(server->seat->seat, event->time_msec,
			event->orientation, event->delta, event->delta_discrete,
			event->source);
}

static void wc_cursor_frame(struct wl_listener *listener, void *data) {
	struct wc_cursor *cursor = wl_container_of(listener, cursor, frame);
	struct wc_server *server = cursor->server;

	wlr_seat_pointer_notify_frame(server->seat->seat);
}

void wc_cursor_init(struct wc_server *server) {
	struct wc_cursor *cursor = calloc(1, sizeof(struct wc_cursor));
	server->cursor = cursor;
	cursor->wlr_cursor = wlr_cursor_create();
	cursor->server = server;

	wlr_cursor_attach_output_layout(cursor->wlr_cursor, server->output_layout);

	cursor->motion.notify = wc_cursor_motion;
	cursor->motion_absolute.notify = wc_cursor_motion_absolute;
	cursor->button.notify = wc_cursor_button;
	cursor->axis.notify = wc_cursor_axis;
	cursor->frame.notify = wc_cursor_frame;

	wl_signal_add(&cursor->wlr_cursor->events.motion, &cursor->motion);
	wl_signal_add(&cursor->wlr_cursor->events.motion_absolute,
			&cursor->motion_absolute);
	wl_signal_add(&cursor->wlr_cursor->events.button, &cursor->button);
	wl_signal_add(&cursor->wlr_cursor->events.axis, &cursor->axis);
	wl_signal_add(&cursor->wlr_cursor->events.frame, &cursor->frame);

	server->xcursor_mgr = wlr_xcursor_manager_create(NULL, 24);
	wlr_xcursor_manager_load(server->xcursor_mgr, 1);
}

void wc_cursor_fini(struct wc_server *server) {
	struct wc_cursor *cursor = server->cursor;

	// NOTE wlroots takes care of this,
	// otherwise this will be a double free.
	// wlr_xcursor_manager_destroy(server->xcursor_mgr);

	wl_list_remove(&cursor->motion.link);
	wl_list_remove(&cursor->motion_absolute.link);
	wl_list_remove(&cursor->button.link);
	wl_list_remove(&cursor->axis.link);
	wl_list_remove(&cursor->frame.link);

	wlr_cursor_destroy(cursor->wlr_cursor);
	cursor->wlr_cursor = NULL;

	free(server->cursor);
	server->cursor = NULL;
}
