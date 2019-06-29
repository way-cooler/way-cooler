#ifndef WC_KEYBOARD_H
#define WC_KEYBOARD_H

#include "wlr/types/wlr_input_device.h"

#include "server.h"

struct wc_keyboard {
	struct wl_list link;
	struct wc_server *server;

	struct wlr_input_device *device;

	struct wl_listener key;
	struct wl_listener modifiers;
	struct wl_listener destroy;
};

void wc_new_keyboard(struct wc_server *server, struct wlr_input_device *device);
void wc_init_keyboards(struct wc_server *server);

#endif  // WC_KEYBOARD_H
