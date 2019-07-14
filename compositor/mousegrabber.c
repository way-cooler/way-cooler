#include "mousegrabber.h"

#include <assert.h>
#include <stdlib.h>

#include <wayland-server.h>
#include <wlr/util/log.h>

#include "server.h"
#include "way-cooler-mousegrabber-unstable-v1-protocol.h"

static void grab_mouse(struct wl_client *client, struct wl_resource *resource) {
	struct wc_mousegrabber *mousegrabber = wl_resource_get_user_data(resource);

	if (mousegrabber->resource != NULL) {
		wl_resource_post_error(resource,
				ZWAY_COOLER_MOUSEGRABBER_ERROR_ALREADY_GRABBED,
				"mouse has already been grabbed");
		return;
	}

	mousegrabber->resource = resource;
	mousegrabber->client = client;

	wlr_log(WLR_DEBUG, "mousegrabber: mouse grabbed");
}

static void release_mouse(
		struct wl_client *client, struct wl_resource *resource) {
	struct wc_mousegrabber *mousegrabber = wl_resource_get_user_data(resource);

	if (mousegrabber->resource != NULL) {
		assert(mousegrabber->client);
	}

	if (mousegrabber->resource == NULL || mousegrabber->client != client) {
		wl_resource_post_error(resource,
				ZWAY_COOLER_MOUSEGRABBER_ERROR_NOT_GRABBED,
				"mouse has not been grabbed by this client");
		return;
	}

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

void wc_mousegrabber_notify_mouse_moved(
		struct wc_server *server, int x, int y) {
	struct wc_mousegrabber *mousegrabber = server->mousegrabber;

	if (mousegrabber == NULL || mousegrabber->resource == NULL) {
		return;
	}

	zway_cooler_mousegrabber_send_mouse_moved(mousegrabber->resource, x, y);
}

void wc_mousegrabber_init(struct wc_server *server) {
	struct wc_mousegrabber *mousegrabber = calloc(1, sizeof(mousegrabber));
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
