#define _POSIX_C_SOURCE 200809L

#include <stdlib.h>

#include "server.h"

#include <wayland-server.h>
#include <wlr/types/wlr_compositor.h>
#include <wlr/backend.h>
#include <wlr/render/wlr_renderer.h>
#include <wlr/types/wlr_cursor.h>
#include <wlr/types/wlr_output.h>
#include <wlr/types/wlr_output_layout.h>
#include <wlr/types/wlr_screencopy_v1.h>
#include <wlr/types/wlr_xcursor_manager.h>

#include "cursor.h"
#include "input.h"
#include "layer_shell.h"
#include "output.h"
#include "seat.h"
#include "view.h"

bool init_server(struct wc_server* server) {
	if (server == NULL) {
		return false;
	}
	server->wl_display = wl_display_create();
	server->wayland_socket = wl_display_add_socket_auto(server->wl_display);
	if (!server->wayland_socket) {
		wlr_backend_destroy(server->backend);
		return false;
	}

	server->backend = wlr_backend_autocreate(server->wl_display, NULL);
	server->renderer = wlr_backend_get_renderer(server->backend);
	wlr_renderer_init_wl_display(server->renderer, server->wl_display);
	wlr_compositor_create(server->wl_display, server->renderer);

    server->screencopy_manager = wlr_screencopy_manager_v1_create(server->wl_display);

	server->cursor_mode = WC_CURSOR_PASSTHROUGH;

	wc_init_seat(server);
	wc_init_output(server);
	wc_init_inputs(server);
	wc_init_views(server);
	wc_init_layers(server);
	wc_init_cursor(server);

	return true;
}

void fini_server(struct wc_server* server) {
    wlr_screencopy_manager_v1_destroy(server->screencopy_manager);
	wl_display_destroy_clients(server->wl_display);
	wl_display_destroy(server->wl_display);
}
