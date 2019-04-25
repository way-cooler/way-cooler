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

#endif//WC_VIEW_H
