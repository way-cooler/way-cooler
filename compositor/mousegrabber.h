#ifndef WC_MOUSEGRABBER_H
#define WC_MOUSEGRABBER_H

#include <wayland-server.h>

#include "server.h"

#define MOUSEGRABBER_VERSION 1

struct wc_mousegrabber {
	struct wl_global *global;
	struct wl_resource *resource;
	struct wl_client *client;
};

void wc_mousegrabber_init(struct wc_server *server);

void wc_mousegrabber_fini(struct wc_server *server);

void wc_mousegrabber_notify_mouse_moved(struct wc_server *server, int x, int y);

#endif  // WC_MOUSEGRABBER_H
