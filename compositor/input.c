#include "input.h"

#include <unistd.h>

#include <wayland-server.h>
#include <wlr/types/wlr_input_device.h>
#include <wlr/util/log.h>

#include "keyboard.h"
#include "pointer.h"
#include "seat.h"
#include "server.h"

static void wc_new_input(struct wl_listener* listener, void* data) {
	struct wc_server *server = wl_container_of(listener, server, new_input);
	struct wlr_input_device *device = data;
	switch (device->type) {
	case WLR_INPUT_DEVICE_KEYBOARD:
		wc_new_keyboard(server, device);
		break;
	case WLR_INPUT_DEVICE_POINTER:
		wc_new_pointer(server, device);
		break;
	default:
		wlr_log(WLR_ERROR, "Device type not supported: %d", device->type);
		break;
	}
	uint32_t caps = WL_SEAT_CAPABILITY_POINTER;
	if (!wl_list_empty(&server->keyboards)) {
		caps |= WL_SEAT_CAPABILITY_KEYBOARD;
	}
	wlr_seat_set_capabilities(server->seat->seat, caps);
}

void wc_init_inputs(struct wc_server* server) {
	server->new_input.notify = wc_new_input;
	wl_signal_add(&server->backend->events.new_input, &server->new_input);

	wc_init_keyboards(server);
	wc_init_pointers(server);
}
