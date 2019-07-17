#include "mousegrabber.h"

#include <assert.h>
#include <stdlib.h>

#include <wayland-server.h>
#include <wlr/types/wlr_output.h>
#include <wlr/util/log.h>

#include "cursor.h"
#include "output.h"
#include "server.h"
#include "way-cooler-mousegrabber-unstable-v1-protocol.h"

static void grab_mouse(struct wl_client *client, struct wl_resource *resource,
		const char *new_cursor_name) {
	struct wc_mousegrabber *mousegrabber = wl_resource_get_user_data(resource);
	struct wc_server *server = mousegrabber->server;
	struct wc_cursor *cursor = server->cursor;

	if (mousegrabber->resource != NULL) {
		wl_resource_post_error(resource,
				ZWAY_COOLER_MOUSEGRABBER_ERROR_ALREADY_GRABBED,
				"mouse has already been grabbed");
		return;
	}

	mousegrabber->resource = resource;
	mousegrabber->client = client;

	server->mouse_grab = true;
	cursor->cursor_mode = WC_CURSOR_PASSTHROUGH;
	wc_cursor_set_compositor_cursor(cursor, new_cursor_name);

	wlr_log(WLR_DEBUG, "mousegrabber: mouse grabbed");
}

static void release_mouse(
		struct wl_client *client, struct wl_resource *resource) {
	struct wc_mousegrabber *mousegrabber = wl_resource_get_user_data(resource);
	struct wc_server *server = mousegrabber->server;
	struct wc_cursor *cursor = server->cursor;

	if (mousegrabber->resource != NULL) {
		assert(mousegrabber->client);
	}

	if (mousegrabber->resource == NULL || mousegrabber->client != client) {
		wl_resource_post_error(resource,
				ZWAY_COOLER_MOUSEGRABBER_ERROR_NOT_GRABBED,
				"mouse has not been grabbed by this client");
		return;
	}

	server->mouse_grab = false;
	wc_cursor_set_compositor_cursor(cursor, NULL);

	// NOTE: Calls our destroy event, which clears client resource pointers.
	wl_resource_destroy(mousegrabber->resource);

	wlr_log(WLR_DEBUG, "mousegrabber: mouse released");
}

static const struct zway_cooler_mousegrabber_interface mousegrabber_impl = {
		.grab_mouse = grab_mouse,
		.release_mouse = release_mouse,
};

static void mousegrabber_handle_resource_destroy(struct wl_resource *resource) {
	struct wc_mousegrabber *mousegrabber = wl_resource_get_user_data(resource);

	if (mousegrabber->resource == resource) {
		mousegrabber->resource = NULL;
		mousegrabber->client = NULL;
	}
}

static void mousegrabber_bind(struct wl_client *wl_client, void *data,
		uint32_t version, uint32_t id) {
	struct wc_mousegrabber *mousegrabber = data;
	struct wl_resource *resource = wl_resource_create(
			wl_client, &zway_cooler_mousegrabber_interface, version, id);
	wl_resource_set_user_data(resource, mousegrabber);

	if (resource == NULL) {
		wl_client_post_no_memory(wl_client);
		return;
	}

	wl_resource_set_implementation(resource, &mousegrabber_impl, mousegrabber,
			mousegrabber_handle_resource_destroy);
}

void wc_mousegrabber_init(struct wc_server *server) {
	struct wc_mousegrabber *mousegrabber = calloc(1, sizeof(mousegrabber));
	mousegrabber->server = server;
	mousegrabber->global = wl_global_create(server->wl_display,
			&zway_cooler_mousegrabber_interface, MOUSEGRABBER_VERSION,
			mousegrabber, mousegrabber_bind);

	server->mousegrabber = mousegrabber;
}

void wc_mousegrabber_fini(struct wc_server *server) {
	wl_list_remove(wl_resource_get_link(server->mousegrabber->resource));
	wl_global_destroy(server->mousegrabber->global);

	free(server->mousegrabber);

	server->mousegrabber = NULL;
}

void wc_mousegrabber_notify_mouse_moved(
		struct wc_mousegrabber *mousegrabber, int x, int y) {
	if (mousegrabber == NULL || mousegrabber->resource == NULL) {
		return;
	}

	zway_cooler_mousegrabber_send_mouse_moved(mousegrabber->resource, x, y);
}

void wc_mousegrabber_notify_mouse_button(struct wc_mousegrabber *mousegrabber,
		int x, int y, struct wlr_event_pointer_button *event) {
	if (mousegrabber == NULL || mousegrabber->resource == NULL) {
		return;
	}
	enum zway_cooler_mousegrabber_button_state pressed =
			event->state == WLR_BUTTON_PRESSED ?
			ZWAY_COOLER_MOUSEGRABBER_BUTTON_STATE_PRESSED :
			ZWAY_COOLER_MOUSEGRABBER_BUTTON_STATE_RELEASED;

	zway_cooler_mousegrabber_send_mouse_button(
			mousegrabber->resource, x, y, pressed, event->button);
}
