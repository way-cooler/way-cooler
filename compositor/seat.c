#include "seat.h"

#include <wlr/types/wlr_seat.h>

#include "cursor.h"
#include "server.h"

static void wc_seat_request_cursor(struct wl_listener* listener, void* data) {
	struct wc_server* server = wl_container_of(listener, server,
			request_set_cursor);
	struct wlr_seat_pointer_request_set_cursor_event* event = data;
	// TODO Check that the client is the same as the focused client
	wlr_cursor_set_surface(server->cursor->wlr_cursor,
			event->surface, event->hotspot_x, event->hotspot_y);
}

void init_seat(struct wc_server* server) {
	server->seat = wlr_seat_create(server->wl_display, "seat0");
	server->request_set_cursor.notify = wc_seat_request_cursor;
	wl_signal_add(&server->seat->events.request_set_cursor,
			&server->request_set_cursor);
}
