#ifndef LAYER_SHELL_H
#define LAYER_SHELL_H

#include <wayland-server.h>

#include "server.h"

struct wc_layer {
	struct wl_list link;
	struct wc_server *server;

	struct wlr_layer_surface_v1 *layer_surface;
	struct wlr_box geo;
	bool mapped;

	struct wl_listener commit;
	struct wl_listener map;
	struct wl_listener unmap;
	struct wl_listener destroy;
};

void wc_init_layers(struct wc_server *server);

// Arrange the layer shells on this output.
void wc_layer_shell_arrange_layers(struct wc_output *output);

#endif  // LAYER_SHELL_H
