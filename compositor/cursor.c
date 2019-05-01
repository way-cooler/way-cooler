#include "cursor.h"

#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <wlr/util/log.h>

#include "output.h"
#include "seat.h"
#include "server.h"
#include "view.h"

static void wc_process_motion(struct wc_server* server, uint32_t time) {
	struct wc_seat* seat = server->seat;
	struct wc_cursor* cursor = server->cursor;
	struct wlr_cursor* wlr_cursor = server->cursor->wlr_cursor;
	struct wc_view* view = server->grabbed_view;
	struct wlr_output* active_output = wlr_output_layout_output_at(
			server->output_layout, wlr_cursor->x, wlr_cursor->y);
	if (active_output == NULL) {
		return;
	}
	switch (server->cursor_mode) {
	case WC_CURSOR_MOVE:
		output_damage_surface(active_output->data, view->xdg_surface->surface,
				view->x, view->y);
		view->x = wlr_cursor->x - server->grab_x;
		view->y = wlr_cursor->y - server->grab_y;
		output_damage_surface(active_output->data, view->xdg_surface->surface,
				view->x, view->y);
		break;
	case WC_CURSOR_RESIZE: {
		double dx = wlr_cursor->x - server->grab_x;
		double dy = wlr_cursor->y - server->grab_y;
		double x = view->x;
		double y = view->y;
		int width = server->grab_width;
		int height = server->grab_height;
		if (server->resize_edges & WLR_EDGE_TOP) {
			y = server->grab_y + dy;
			height -= dy;
			if (height < 1) {
				y += height;
			}
		} else if (server->resize_edges & WLR_EDGE_BOTTOM) {
			height += dy;
		}
		if (server->resize_edges & WLR_EDGE_LEFT) {
			x = server->grab_x + dx;
			width -= dx;
			if (width < 1) {
				x += width;
			}
		} else if (server->resize_edges & WLR_EDGE_RIGHT) {
			width += dx;
		}
		view->x = x;
		view->y = y;
		wlr_xdg_toplevel_set_size(view->xdg_surface, width, height);
		break;
	}
	case WC_CURSOR_PASSTHROUGH: {
		double sx, sy;
		struct wlr_surface* surface = NULL;
		struct wc_view* view = wc_view_at(server,
				wlr_cursor->x, wlr_cursor->y, &sx, &sy, &surface);
		bool cursor_image_different = !cursor->image || strcmp(cursor->image, "left_ptr") != 0;
		if (!view && cursor_image_different) {
			cursor->image = "left_ptr";
			wlr_xcursor_manager_set_cursor_image(server->xcursor_mgr, "left_ptr",
					cursor->wlr_cursor);
		}
		wc_seat_update_surface_focus(seat, surface, sx, sy, time);
		break;
	}
	}

	if (server->active_output->output != active_output) {
		struct wc_output* output_;
		wl_list_for_each(output_, &server->outputs, link) {
			if (output_->output == active_output) {
				server->active_output = output_;
				break;
			}
		}
	}
}

static void wc_cursor_motion(struct wl_listener* listener, void* data) {
	struct wc_cursor *cursor = wl_container_of(listener, cursor, motion);
	struct wlr_event_pointer_motion *event = data;
	wlr_cursor_move(cursor->wlr_cursor, event->device,
			event->delta_x, event->delta_y);
	wc_process_motion(cursor->server, event->time_msec);
}

static void wc_cursor_motion_absolute(struct wl_listener* listener, void* data) {
	struct wc_cursor *cursor = wl_container_of(listener, cursor,
			motion_absolute);
	struct wlr_event_pointer_motion_absolute *event = data;
	wlr_cursor_warp_absolute(cursor->wlr_cursor, event->device, event->x, event->y);
	wc_process_motion(cursor->server, event->time_msec);
}

static void wc_cursor_button(struct wl_listener* listener, void* data) {
	struct wc_cursor* cursor = wl_container_of(listener, cursor, button);
	struct wc_server* server = cursor->server;
	struct wlr_event_pointer_button* event = data;
	wlr_seat_pointer_notify_button(server->seat->seat,
			event->time_msec, event->button, event->state);

	double sx, sy;
	struct wlr_surface* surface = NULL;
	struct wc_view* view = wc_view_at(server,
			cursor->wlr_cursor->x, cursor->wlr_cursor->y, &sx, &sy, &surface);
	if (event->state == WLR_BUTTON_RELEASED) {
		server->cursor_mode = WC_CURSOR_PASSTHROUGH;
	} else if (view) {
		wc_focus_view(view);
	}
}

static void wc_cursor_axis(struct wl_listener* listener, void* data) {
	struct wc_cursor* cursor = wl_container_of(listener, cursor, axis);
	struct wc_server* server = cursor->server;
	struct wlr_event_pointer_axis* event = data;
	wlr_seat_pointer_notify_axis(server->seat->seat,
			event->time_msec, event->orientation, event->delta,
			event->delta_discrete, event->source);
}

static void wc_cursor_frame(struct wl_listener* listener, void* data) {
	struct wc_cursor* cursor = wl_container_of(listener, cursor, frame);
	struct wc_server* server = cursor->server;
	wlr_seat_pointer_notify_frame(server->seat->seat);
}

void wc_init_cursor(struct wc_server* server) {
	struct wc_cursor* cursor = calloc(1, sizeof(struct wc_cursor));
	server->cursor = cursor;
	cursor->wlr_cursor = wlr_cursor_create();
	cursor->server = server;

	wlr_cursor_attach_output_layout(cursor->wlr_cursor, server->output_layout);
	cursor->motion.notify = wc_cursor_motion;
	wl_signal_add(&cursor->wlr_cursor->events.motion,
			&cursor->motion);
	cursor->motion_absolute.notify = wc_cursor_motion_absolute;
	wl_signal_add(&cursor->wlr_cursor->events.motion_absolute,
			&cursor->motion_absolute);
	cursor->button.notify = wc_cursor_button;
	wl_signal_add(&cursor->wlr_cursor->events.button,
			&cursor->button);
	cursor->axis.notify = wc_cursor_axis;
	wl_signal_add(&cursor->wlr_cursor->events.axis,
			&cursor->axis);
	cursor->frame.notify = wc_cursor_frame;
	wl_signal_add(&cursor->wlr_cursor->events.frame,
			&cursor->frame);

	server->xcursor_mgr = wlr_xcursor_manager_create(NULL, 24);
	wlr_xcursor_manager_load(server->xcursor_mgr, 1);
}
