#ifndef WC_OUTPUT_H
#define WC_OUTPUT_H

#include <wayland-server.h>

#include "server.h"

struct wc_output {
	struct wl_list link;
	struct wc_server* server;

	struct wlr_output* output;

	struct wl_listener frame;
};

void wc_output_frame(struct wl_listener* listener, void* data);
void init_output(struct wc_server* server);

#endif // WC_OUTPUT_H
