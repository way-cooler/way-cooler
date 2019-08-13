#ifndef WC_KEYBINDINGS_H
#define WC_KEYBINDINGS_H

#include <stdint.h>
#include <wayland-server.h>

#include "server.h"
#include "xkb_hash_set.h"

#define KEYBINDINGS_VERSION 1

struct wc_keybindings {
	struct wc_server *server;

	struct xkb_hash_set *registered_keys;

	struct wl_global *global;
	struct wl_resource *resource;
	struct wl_client *client;
};

void wc_keybindings_init(struct wc_server *server);

void wc_keybindings_fini(struct wc_server *server);

/*
 * Checks if the key is registered as a keybinding and, if so, sends it to the
 * registered keybindings client.
 *
 * If the key is registered true is returned.
 *
 * Mods is expected to be all mods that are either depressed, latched, or
 * locked.
 */
bool wc_keybindings_notify_key_if_registered(struct wc_keybindings *keybindings,
		uint32_t key_code, xkb_mod_mask_t key_mask, bool pressed,
		uint32_t time);

/*
 * Clears the stored keybindings, meaning those keys will no longer be filtered
 * from other clients.
 */

void wc_keybindings_clear_keys(struct wc_keybindings *keybindings);

#endif  // WC_KEYBINDINGS_H
