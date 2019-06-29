#define _POSIX_C_SOURCE 200809L

#include <stdlib.h>

#include "server.h"

#include <wayland-server.h>
#include <wlr/backend.h>
#include <wlr/render/wlr_renderer.h>
#include <wlr/types/wlr_compositor.h>
#include <wlr/types/wlr_cursor.h>
#include <wlr/types/wlr_data_device.h>
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

bool init_server(struct wc_server *server) {
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
	server->compositor =
			wlr_compositor_create(server->wl_display, server->renderer);
	if (server->compositor == NULL) {
		return false;
	}

	server->screencopy_manager =
			wlr_screencopy_manager_v1_create(server->wl_display);
	server->data_device_manager =
			wlr_data_device_manager_create(server->wl_display);

	wc_seat_init(server);
	wc_output_init(server);
	wc_inputs_init(server);
	wc_views_init(server);
	wc_layers_init(server);
	wc_cursor_init(server);

	return true;
}

void fini_server(struct wc_server *server) {
	wc_seat_fini(server);
	wc_output_fini(server);
	wc_inputs_fini(server);
	wc_views_fini(server);
	wc_layers_fini(server);
	wc_cursor_fini(server);

	wlr_screencopy_manager_v1_destroy(server->screencopy_manager);
	wlr_data_device_manager_destroy(server->data_device_manager);

	wlr_compositor_destroy(server->compositor);

	wl_display_destroy_clients(server->wl_display);
	wl_display_destroy(server->wl_display);
}
