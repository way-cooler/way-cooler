#include "view.h"

#include <stdlib.h>

#include <wayland-server.h>
#include <wlr/types/wlr_surface.h>
#include <wlr/util/log.h>

#include "cursor.h"
#include "layer_shell.h"
#include "seat.h"
#include "server.h"
#include "xdg.h"


void wc_view_get_outputs(struct wlr_output_layout* layout, struct wc_view* view,
		struct wlr_output** out_outputs) {
	int width, height, x, y;
	switch (view->surface_type) {
	case WC_XDG:
		x = view->x;
		y = view->y;
		height = view->xdg_surface->geometry.height;
		width = view->xdg_surface->geometry.width;
	}
	int next_index = 0;
	// top left
	out_outputs[next_index++] =
		wlr_output_layout_output_at(layout, x, y);
	// top right
	out_outputs[next_index++] =
		wlr_output_layout_output_at(layout, x + width, y + height);
	// bottom left
	out_outputs[next_index++] =
		wlr_output_layout_output_at(layout, x, y + height);
	// bottom right
	out_outputs[next_index++] =
		wlr_output_layout_output_at(layout, x + width, y);
}

struct wlr_surface* wc_view_surface(struct wc_view* view) {
	switch (view->surface_type) {
	case WC_XDG:
		return view->xdg_surface->surface;
	default:
		return NULL;
	}
}

static bool wc_is_view_at(struct wc_view* view, double lx, double ly,
		double* out_sx, double* out_sy, struct wlr_surface** out_surface) {
	double view_sx = lx - view->x;
	double view_sy = ly - view->y;

	switch (view->surface_type) {
	case WC_XDG:
		*out_surface = wlr_xdg_surface_surface_at(
				view->xdg_surface, view_sx, view_sy, out_sx, out_sy);
		break;
	}
	return *out_surface != NULL;
}

struct wc_view* wc_view_at(struct wc_server* server, double lx, double ly,
		double* out_sx, double* out_sy, struct wlr_surface** out_surface) {
	struct wc_view* view;
	wl_list_for_each(view, &server->views, link) {
		if (wc_is_view_at(view, lx, ly, out_sx, out_sy, out_surface)) {
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
	struct wlr_surface* surface = wc_view_surface(view);
	struct wlr_seat* seat = server->seat->seat;
	struct wlr_surface* prev_surface =
		server->seat->seat->keyboard_state.focused_surface;
	if (prev_surface == surface) {
		return;
	}
	if (prev_surface && wlr_surface_is_xdg_surface(prev_surface)) {
		struct wlr_xdg_surface* previous =
			wlr_xdg_surface_from_wlr_surface(prev_surface);
		wlr_xdg_toplevel_set_activated(previous, false);
	}
	/* Move the view to the front */
	if (view->surface_type == WC_XDG) {
		wl_list_remove(&view->link);
		wl_list_insert(&server->views, &view->link);
	}
	if (view->surface_type == WC_XDG) {
		wlr_xdg_toplevel_set_activated(view->xdg_surface, true);
	}

	struct wlr_keyboard* keyboard = wlr_seat_get_keyboard(seat);
	wlr_seat_keyboard_notify_enter(seat, surface, keyboard->keycodes,
			keyboard->num_keycodes, &keyboard->modifiers);
}

void wc_init_views(struct wc_server* server) {
	wl_list_init(&server->views);
	wc_init_xdg(server);
}
