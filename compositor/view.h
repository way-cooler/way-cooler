#ifndef WC_VIEW_H
#define WC_VIEW_H

#include <wayland-server.h>
#include <wlr/types/wlr_xdg_shell.h>

#include "server.h"

struct wc_view {
	struct wl_list link;
	struct wc_server *server;

	// TODO This should be abstract over surfaces of all kinds (xwayland, layer shell)
	struct wlr_xdg_surface *xdg_surface;
	bool mapped;
	int x, y;

	struct wl_listener map;
	struct wl_listener unmap;
	struct wl_listener destroy;
	struct wl_listener request_move;
	struct wl_listener request_resize;
};

void init_views(struct wc_server* server);

// Finds the topmost (assuming server->views is top-to-bottom) view at the
// specified output layout coordinates. If one cannot be found NULL is returned.
//
// If a view is found the surface coordinates are stored in out_sx and out_sy.
struct wc_view* wc_view_at(struct wc_server* server, double lx, double ly,
		double* out_sx, double* out_sy);

// Focuses on a view. Automatically un-focuses the previous view.
void wc_focus_view(struct wc_view* view);

#endif//WC_VIEW_H
