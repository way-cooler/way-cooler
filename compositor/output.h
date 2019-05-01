#ifndef WC_OUTPUT_H
#define WC_OUTPUT_H

#include <wayland-server.h>
#include <wlr/types/wlr_box.h>
#include <wlr/types/wlr_output.h>
#include <wlr/types/wlr_output_damage.h>

#include "server.h"

struct wc_output {
	struct wl_list link;
	struct wc_server* server;

	struct wlr_output* output;
	struct wlr_output_damage* damage;

	struct wlr_box usable_area;
	struct wl_list layers[4];

	struct wl_listener destroy;
	struct wl_listener frame;
};

void wc_init_output(struct wc_server* server);

// Gets the output that was last active (e.g. last had user activity).
//
// If there are no outputs, NULL is returned. If there has been no activity,
// the first output in the list is returned.
struct wc_output* wc_get_active_output(struct wc_server* server);

/// Damages the entire surface which is at the given output coordinates.
void output_damage_surface(struct wc_output* output, struct wlr_surface* surface,
		double ox, double oy);

#endif // WC_OUTPUT_H
