#include "xdg.h"

#include <stdlib.h>

#include <wayland-server.h>
#include <wlr/types/wlr_xdg_shell.h>
#include <wlr/util/log.h>

#include "cursor.h"
#include "output.h"
#include "seat.h"
#include "server.h"
#include "view.h"

static void wc_xdg_surface_map(struct wl_listener* listener, void* data) {
	struct wc_view* view = wl_container_of(listener, view, map);
	view->mapped = true;
	wc_focus_view(view);
	// TODO FIXME This can return null (and every other place).
	// Ask on irc how to fix this up because it's broken
	struct wc_output* output =
		wc_view_get_output(view->server->output_layout, view);
	if (output != NULL) {
		output_damage_surface(output, view->xdg_surface->surface,
				view->x, view->y);
	}
}

static void wc_xdg_surface_unmap(struct wl_listener* listener, void* data) {
	struct wc_view* view = wl_container_of(listener, view, unmap);
	view->mapped = false;
	struct wc_output* output =
		wc_view_get_output(view->server->output_layout, view);

	if (output != NULL) {
		output_damage_surface(output, view->xdg_surface->surface,
				view->x, view->y);
	}
}

static void wc_xdg_surface_commit(struct wl_listener* listener, void* data) {
	struct wc_view* view = wl_container_of(listener, view, commit);
	wlr_log(WLR_ERROR, "committing %d",view->mapped);
	if (!view->mapped) {
		return;
	}

	// TODO Damage only what has changed
	struct wc_output* output =
		wc_view_get_output(view->server->output_layout, view);

	if (output) {
		output_damage_surface(output, view->xdg_surface->surface,
				view->x, view->y);
	}
}

static void wc_xdg_surface_destroy(struct wl_listener* listener, void* data) {
	struct wc_view* view = wl_container_of(listener, view, destroy);
	wl_list_remove(&view->link);

	wl_list_remove(&view->map.link);
	wl_list_remove(&view->unmap.link);
	wl_list_remove(&view->commit.link);
	wl_list_remove(&view->request_move.link);
	wl_list_remove(&view->request_resize.link);
	wl_list_remove(&view->destroy.link);

	free(view);
}

static void wc_xdg_toplevel_request_move(struct wl_listener* listener, void* data) {
	struct wc_view* view = wl_container_of(listener, view, request_move);
	struct wc_server* server = view->server;
	struct wlr_cursor* wlr_cursor = server->cursor->wlr_cursor;
	struct wlr_surface* focused_surface =
		server->seat->seat->pointer_state.focused_surface;
	struct wlr_surface* surface = wc_view_surface(view);
	if (surface != focused_surface) {
		return;
	}
	server->grabbed_view = view;
	server->cursor_mode = WC_CURSOR_MOVE;
	struct wlr_box geo_box;
	wlr_xdg_surface_get_geometry(view->xdg_surface, &geo_box);
	server->grab_x = wlr_cursor->x - view->x;
	server->grab_y = wlr_cursor->y - view->y;
	server->grab_width = geo_box.width;
	server->grab_height = geo_box.height;
}

static void wc_xdg_toplevel_request_resize(struct wl_listener* listener, void* data) {
	struct wc_view* view = wl_container_of(listener, view, request_resize);
	struct wlr_xdg_toplevel_resize_event *event = data;
	struct wc_server* server = view->server;
	struct wlr_cursor* wlr_cursor = server->cursor->wlr_cursor;
	struct wlr_surface* focused_surface =
		server->seat->seat->pointer_state.focused_surface;
	struct wlr_surface* surface = wc_view_surface(view);
	if (surface != focused_surface) {
		return;
	}
	server->grabbed_view = view;
	server->cursor_mode = WC_CURSOR_RESIZE;
	struct wlr_box geo_box;
	wlr_xdg_surface_get_geometry(view->xdg_surface, &geo_box);
	server->grab_x = wlr_cursor->x + geo_box.x;
	server->grab_y = wlr_cursor->y + geo_box.y;
	server->grab_width = geo_box.width;
	server->grab_height = geo_box.height;
	server->resize_edges = event->edges;
}

static void wc_xdg_new_surface(struct wl_listener* listener, void* data) {
	struct wc_server* server = wl_container_of(listener, server, new_xdg_surface);
	struct wlr_xdg_surface* xdg_surface = data;
	if (xdg_surface->role != WLR_XDG_SURFACE_ROLE_TOPLEVEL) {
		return;
	}

	struct wc_view* view = calloc(1, sizeof(struct wc_view));
	view->server = server;
	view->xdg_surface = xdg_surface;
	view->surface_type = WC_XDG;

	view->map.notify = wc_xdg_surface_map;
	wl_signal_add(&xdg_surface->events.map, &view->map);
	view->unmap.notify = wc_xdg_surface_unmap;
	wl_signal_add(&xdg_surface->events.unmap, &view->unmap);
	view->commit.notify = wc_xdg_surface_commit;
	wl_signal_add(&xdg_surface->surface->events.commit, &view->commit);
	view->destroy.notify = wc_xdg_surface_destroy;
	wl_signal_add(&xdg_surface->events.destroy, &view->destroy);

	struct wlr_xdg_toplevel *toplevel = xdg_surface->toplevel;
	view->request_move.notify = wc_xdg_toplevel_request_move;
	wl_signal_add(&toplevel->events.request_move, &view->request_move);
	view->request_resize.notify = wc_xdg_toplevel_request_resize;
	wl_signal_add(&toplevel->events.request_resize, &view->request_resize);

	wl_list_insert(&server->views, &view->link);
}

void wc_init_xdg(struct wc_server* server) {
	server->xdg_shell = wlr_xdg_shell_create(server->wl_display);
	server->new_xdg_surface.notify = wc_xdg_new_surface;
	wl_signal_add(&server->xdg_shell->events.new_surface,
			&server->new_xdg_surface);
}
