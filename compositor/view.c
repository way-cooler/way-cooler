#include "view.h"

#include <stdlib.h>

#include <wayland-server.h>
#include <wlr/types/wlr_surface.h>
#include <wlr/types/wlr_xdg_shell.h>

#include "cursor.h"
#include "server.h"

static bool is_view_at(struct wc_view* view, double lx, double ly,
		double* out_sx, double* out_sy, struct wlr_surface** out_surface) {
	double view_sx = lx - view->x;
	double view_sy = ly - view->y;

	*out_surface = wlr_xdg_surface_surface_at(
			view->xdg_surface, view_sx, view_sy, out_sx, out_sy);
	return *out_surface != NULL;
}

struct wc_view* wc_view_at(struct wc_server* server, double lx, double ly,
		double* out_sx, double* out_sy, struct wlr_surface** out_surface) {
	struct wc_view* view;
	wl_list_for_each(view, &server->views, link) {
		if (is_view_at(view, lx, ly, out_sx, out_sy, out_surface)) {
			return view;
		}
	}
	return NULL;
}

void wc_focus_view(struct wc_view* view) {
	if (view == NULL) {
		return;
	}
	struct wc_server* server = view->server;
	struct wlr_surface* surface = view->xdg_surface->surface;
	struct wlr_seat* seat = server->seat;
	struct wlr_surface* prev_surface = server->seat->keyboard_state.focused_surface;
	if (prev_surface == surface) {
		return;
	}
	if (prev_surface) {
		struct wlr_xdg_surface* previous = wlr_xdg_surface_from_wlr_surface(
				seat->keyboard_state.focused_surface);
		wlr_xdg_toplevel_set_activated(previous, false);
	}
	/* Move the view to the front */
	wl_list_remove(&view->link);
	wl_list_insert(&server->views, &view->link);
	wlr_xdg_toplevel_set_activated(view->xdg_surface, true);

	struct wlr_keyboard* keyboard = wlr_seat_get_keyboard(seat);
	wlr_seat_keyboard_notify_enter(seat, view->xdg_surface->surface,
			keyboard->keycodes, keyboard->num_keycodes, &keyboard->modifiers);
}

static void wc_xdg_surface_map(struct wl_listener* listener, void* data) {
	struct wc_view* view = wl_container_of(listener, view, map);
	view->mapped = true;
	wc_focus_view(view);
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
	struct wc_view* view = wl_container_of(listener, view, request_move);
	struct wc_server* server = view->server;
	struct wlr_cursor* wlr_cursor = server->cursor->wlr_cursor;
	struct wlr_surface* focused_surface =
		server->seat->pointer_state.focused_surface;
	if (view->xdg_surface->surface != focused_surface) {
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
		server->seat->pointer_state.focused_surface;
	if (view->xdg_surface->surface != focused_surface) {
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
