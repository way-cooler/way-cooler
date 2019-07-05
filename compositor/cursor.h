#ifndef WC_CURSOR_H
#define WC_CURSOR_H

#include <wayland-server.h>
#include <wlr/types/wlr_box.h>
#include <wlr/types/wlr_cursor.h>

enum wc_cursor_mode {
	WC_CURSOR_PASSTHROUGH = 0,
	WC_CURSOR_MOVE,
	WC_CURSOR_RESIZE,
};

struct wc_cursor {
	struct wc_server *server;
	struct wlr_cursor *wlr_cursor;

	char *image;

	enum wc_cursor_mode cursor_mode;
	/*
	 * Original location data of a view when it is grabbed. This is
	 * used in calculations when resizing and moving it from
	 * the original location.
	 *
	 * depending on mode, these may or may not be valid
	 */
	struct {
		struct wc_view *view;
		// Original coordinates of where the cursor was.
		int original_x, original_y;
		struct wlr_box original_view_geo;
		uint32_t resize_edges;
	} grabbed;

	struct wl_listener motion;
	struct wl_listener motion_absolute;
	struct wl_listener button;
	struct wl_listener axis;
	struct wl_listener frame;
};

void wc_cursor_init(struct wc_server *server);

void wc_cursor_fini(struct wc_server *server);

#endif  // WC_CURSOR_H
