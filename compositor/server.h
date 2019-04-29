#ifndef WC_SERVER_H
#define WC_SERVER_H

#include <wayland-server.h>
#include <wlr/types/wlr_compositor.h>
#include <wlr/backend.h>
#include <wlr/render/wlr_renderer.h>
#include <wlr/types/wlr_cursor.h>
#include <wlr/types/wlr_seat.h>
#include <wlr/types/wlr_output.h>
#include <wlr/types/wlr_output_layout.h>
#include <wlr/types/wlr_xcursor_manager.h>

enum wc_cursor_mode {
	WC_CURSOR_PASSTHROUGH,
	WC_CURSOR_MOVE,
	WC_CURSOR_RESIZE,
};

struct wc_server {
	const char* wayland_socket;
	struct wl_display* wl_display;
	struct wlr_backend* backend;
	struct wlr_renderer* renderer;

	struct wlr_xcursor_manager* xcursor_mgr;
	struct wc_cursor *cursor;
	enum wc_cursor_mode cursor_mode;
	struct wc_view* grabbed_view;
	double grab_x, grab_y;
    int grab_width, grab_height;
	uint32_t resize_edges;

	struct wlr_seat* seat;
	struct wl_listener request_set_cursor;

	struct wl_list keyboards;
	struct wl_list pointers;
	struct wl_listener new_input;

	struct wlr_output_layout* output_layout;
	struct wc_output* active_output;
	struct wl_list outputs;
	struct wl_listener new_output;

	struct wlr_xdg_shell* xdg_shell;
	struct wl_listener new_xdg_surface;
	struct wl_list views;
};

bool init_server(struct wc_server* server);
void fini_server(struct wc_server* server);


#endif // WC_SERVER_H
