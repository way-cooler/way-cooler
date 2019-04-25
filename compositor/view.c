#include "view.h"

#include <stdlib.h>

#include <wayland-server.h>
#include <wlr/types/wlr_xdg_shell.h>

#include "server.h"

static void wc_xdg_surface_map(struct wl_listener* listener, void* data) {
	struct wc_view* view = wl_container_of(listener, view, map);
	view->mapped = true;
	// TODO focus the view
}

static void wc_xdg_surface_unmap(struct wl_listener* listener, void* data) {
	struct wc_view* view = wl_container_of(listener, view, unmap);
	view->mapped = false;
}

static void wc_xdg_surface_destroy(struct wl_listener* listener, void* data) {
	struct wc_view* view = wl_container_of(listener, view, destroy);
	wl_list_remove(&view->link);
	free(view);
}

static void wc_xdg_toplevel_request_move(struct wl_listener* listener, void* data) {
	// TODO
}

static void wc_xdg_toplevel_request_resize(struct wl_listener* listener, void* data) {
	// TODO
}

static void wc_new_xdg_surface(struct wl_listener* listener, void* data) {
	struct wc_server* server = wl_container_of(listener, server, new_xdg_surface);
	struct wlr_xdg_surface* xdg_surface = data;
	if (xdg_surface->role != WLR_XDG_SURFACE_ROLE_TOPLEVEL) {
		return;
	}

	struct wc_view* view = calloc(1, sizeof(struct wc_view));
	view->server = server;
	view->xdg_surface = xdg_surface;

	view->map.notify = wc_xdg_surface_map;
	wl_signal_add(&xdg_surface->events.map, &view->map);
	view->unmap.notify = wc_xdg_surface_unmap;
	wl_signal_add(&xdg_surface->events.unmap, &view->unmap);
	view->destroy.notify = wc_xdg_surface_destroy;
	wl_signal_add(&xdg_surface->events.destroy, &view->destroy);

	struct wlr_xdg_toplevel *toplevel = xdg_surface->toplevel;
	view->request_move.notify = wc_xdg_toplevel_request_move;
	wl_signal_add(&toplevel->events.request_move, &view->request_move);
	view->request_resize.notify = wc_xdg_toplevel_request_resize;
	wl_signal_add(&toplevel->events.request_resize, &view->request_resize);

	wl_list_insert(&server->views, &view->link);
}

void init_views(struct wc_server* server) {
	wl_list_init(&server->views);
	server->xdg_shell = wlr_xdg_shell_create(server->wl_display);
	server->new_xdg_surface.notify = wc_new_xdg_surface;
	wl_signal_add(&server->xdg_shell->events.new_surface,
				  &server->new_xdg_surface);
}
