#include "cursor.h"

#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "server.h"

static void wc_process_motion(struct wc_server* server, struct wc_cursor* cursor) {
	// TODO Only do this if no client underneath pointer
	if (!cursor->image || strcmp(cursor->image, "left_ptr") != 0) {
		cursor->image = "left_ptr";
		wlr_xcursor_manager_set_cursor_image(server->xcursor_mgr, "left_ptr",
				cursor->wlr_cursor);
	}
	// TODO Process entering and other things
}

static void wc_cursor_motion(struct wl_listener* listener, void* data) {
	struct wc_cursor *cursor = wl_container_of(listener, cursor, motion);
	struct wlr_event_pointer_motion *event = data;
	wlr_cursor_move(cursor->wlr_cursor, event->device,
			event->delta_x, event->delta_y);
	wc_process_motion(cursor->server, cursor);
}

static void wc_cursor_motion_absolute(struct wl_listener* listener, void* data) {
	struct wc_cursor *cursor = wl_container_of(listener, cursor,
			motion_absolute);
	struct wlr_event_pointer_motion_absolute *event = data;
	wlr_cursor_warp_absolute(cursor->wlr_cursor, event->device, event->x, event->y);
	wc_process_motion(cursor->server, cursor);
}

static void wc_cursor_button(struct wl_listener* listener, void* data) {
	struct wc_cursor* cursor = wl_container_of(listener, cursor, button);
	struct wc_server* server = cursor->server;
	struct wlr_event_pointer_button* event = data;
	wlr_seat_pointer_notify_button(server->seat,
			event->time_msec, event->button, event->state);
	// TODO Click and focus and stuff
}

static void wc_cursor_axis(struct wl_listener* listener, void* data) {
	struct wc_cursor* cursor = wl_container_of(listener, cursor, axis);
	struct wc_server* server = cursor->server;
	struct wlr_event_pointer_axis* event = data;
	wlr_seat_pointer_notify_axis(server->seat,
			event->time_msec, event->orientation, event->delta,
			event->delta_discrete, event->source);
}

static void wc_cursor_frame(struct wl_listener* listener, void* data) {
	struct wc_cursor* cursor = wl_container_of(listener, cursor, frame);
	struct wc_server* server = cursor->server;
	wlr_seat_pointer_notify_frame(server->seat);
}

void init_cursor(struct wc_server* server) {
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
