#ifndef WC_VIEW_H
#define WC_VIEW_H

#include <wayland-server.h>
#include <wlr/types/wlr_surface.h>
#include <wlr/types/wlr_layer_shell_v1.h>
#include <wlr/types/wlr_xdg_shell.h>

#include "server.h"

enum wc_surface_type {
	WC_XDG,
};

struct wc_view {
	struct wl_list link;
	struct wc_server* server;

	enum wc_surface_type surface_type;
	union {
		struct wlr_xdg_surface* xdg_surface;
	};

	bool mapped;
	int x, y;

	// These variables are layer surface specific
	struct wlr_box wc_layer_geo;

	struct wl_listener map;
	struct wl_listener unmap;
	struct wl_listener commit;
	struct wl_listener destroy;
	struct wl_listener request_move;
	struct wl_listener request_resize;
};

void wc_init_views(struct wc_server* server);

// Get the main surface associated with the view.
struct wlr_surface* wc_view_surface(struct wc_view* view);

// Finds the topmost (assuming server->views is top-to-bottom) view at the
// specified output layout coordinates. If one cannot be found NULL is returned.
//
// The out_surface parameter is the surface at that point. Note that this might be
// a subsurface of the view and thus that is why it is returned.
//
// If a view is found the surface coordinates are stored in out_sx and out_sy.
struct wc_view* wc_view_at(struct wc_server* server, double lx, double ly,
		double* out_sx, double* out_sy, struct wlr_surface** out_surface);

// Focuses on a view. Automatically un-focuses the previous view.
void wc_focus_view(struct wc_view* view);

// Get the output that the view is on.
//
// NULL could be returned if none of the corners or center is on an output.
struct wc_output* wc_view_get_output(struct wlr_output_layout* layout,
		struct wc_view* view);

#endif//WC_VIEW_H
