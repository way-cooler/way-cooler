#ifndef WC_SEAT_H
#define WC_SEAT_H

#include <wayland-server.h>

#include "server.h"

struct wc_seat {
	struct wl_list link;
	struct wc_server* server;
};

void wc_init_seat(struct wc_server* server);

#endif//WC_SEAT_H
