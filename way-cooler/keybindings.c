#include "keybindings.h"

#include <stdint.h>
#include <stdlib.h>
#include <wayland-server.h>

#include "seat.h"
#include "server.h"
#include "way-cooler-keybindings-unstable-v1-protocol.h"
#include "xkb_hash_set.h"

static void register_key(struct wl_client *client, struct wl_resource *resource,
		uint32_t key, uint32_t mods) {
	struct wc_keybindings *keybindings = wl_resource_get_user_data(resource);
	struct xkb_hash_set *registered_keys = keybindings->registered_keys;

	xkb_hash_set_add_entry(registered_keys, key, mods);
}

static void clear_keys(struct wl_client *client, struct wl_resource *resource) {
	struct wc_keybindings *keybindings = wl_resource_get_user_data(resource);
	wc_keybindings_clear_keys(keybindings);
}

static const struct zway_cooler_keybindings_interface keybindings_impl = {
		.register_key = register_key,
		.clear_keys = clear_keys,
};

static void keybindings_handle_resource_destroy(struct wl_resource *resource) {
	struct wc_keybindings *keybindings = wl_resource_get_user_data(resource);

	if (keybindings->resource == resource) {
		keybindings->resource = NULL;
		keybindings->client = NULL;
	}
}

static void keybindings_bind(
		struct wl_client *client, void *data, uint32_t version, uint32_t id) {
	struct wc_keybindings *keybindings = data;
	struct wl_resource *resource = wl_resource_create(
			client, &zway_cooler_keybindings_interface, version, id);
	wl_resource_set_user_data(resource, keybindings);

	if (resource == NULL) {
		wl_client_post_no_memory(client);
		return;
	}

	keybindings->resource = resource;

	wl_resource_set_implementation(resource, &keybindings_impl, keybindings,
			keybindings_handle_resource_destroy);
}

void wc_keybindings_init(struct wc_server *server) {
	struct wc_keybindings *keybindings =
			calloc(1, sizeof(struct wc_keybindings));
	keybindings->server = server;
	keybindings->global = wl_global_create(server->wl_display,
			&zway_cooler_keybindings_interface, KEYBINDINGS_VERSION,
			keybindings, keybindings_bind);

	keybindings->registered_keys = calloc(1, sizeof(struct xkb_hash_set));

	server->keybindings = keybindings;
}

void wc_keybindings_fini(struct wc_server *server) {
	wl_global_destroy(server->keybindings->global);

	wc_keybindings_clear_keys(server->keybindings);

	if (server->keybindings->registered_keys) {
		free(server->keybindings->registered_keys);
	}

	free(server->keybindings);

	server->keybindings = NULL;
}

void wc_keybindings_clear_keys(struct wc_keybindings *keybindings) {
	struct xkb_hash_set *registered_keys = keybindings->registered_keys;
	xkb_hash_set_clear(registered_keys);
}

bool wc_keybindings_notify_key_if_registered(struct wc_keybindings *keybindings,
		uint32_t key_code, xkb_mod_mask_t key_mask, bool pressed,
		uint32_t time) {
	struct wc_server *server = keybindings->server;

	if (keybindings->resource == NULL) {
		return false;
	}

	struct xkb_hash_set *registered_keys = keybindings->registered_keys;

	xkb_mod_mask_t out_mask = 0;
	bool present = xkb_hash_set_get_entry(registered_keys, key_code, &out_mask);
	if (present) {
		present = (out_mask & key_mask);
	}

	enum zway_cooler_keybindings_key_state press_state = pressed ?
			ZWAY_COOLER_KEYBINDINGS_KEY_STATE_PRESSED :
			ZWAY_COOLER_KEYBINDINGS_KEY_STATE_RELEASED;
	zway_cooler_keybindings_send_key(
			keybindings->resource, time, key_code, press_state, key_mask);

	struct wlr_seat_client *focused_client =
			server->seat->seat->keyboard_state.focused_client;
	if (!present && focused_client)
		present = present || focused_client->client == keybindings->client;

	return present;
}
