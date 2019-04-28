#ifndef WC_CURSOR_H
#define WC_CURSOR_H

#include <wayland-server.h>
#include <wlr/types/wlr_cursor.h>

struct wc_cursor {
	struct wc_server* server;
	struct wlr_cursor* wlr_cursor;

	char* image;

	struct wl_listener motion;
	struct wl_listener motion_absolute;
	struct wl_listener button;
	struct wl_listener axis;
	struct wl_listener frame;
};

void init_cursor(struct wc_server* server);

#endif//WC_CURSOR_H
