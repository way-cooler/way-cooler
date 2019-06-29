#include "pointer.h"

#include <stdlib.h>

#include <wayland-server.h>
#include <wlr/types/wlr_input_device.h>

#include "cursor.h"

static void wc_pointer_removed(struct wl_listener *listener, void *data) {
	struct wc_pointer *pointer = wl_container_of(listener, pointer, destroy);
	wl_list_remove(&pointer->link);

	wl_list_remove(&pointer->destroy.link);

	free(pointer);
}

void wc_new_pointer(struct wc_server *server, struct wlr_input_device *device) {
	struct wc_pointer *pointer = calloc(1, sizeof(struct wc_pointer));
	pointer->server = server;
	pointer->device = device;
	pointer->destroy.notify = wc_pointer_removed;
	wl_signal_add(&device->events.destroy, &pointer->destroy);

	wl_list_insert(&server->pointers, &pointer->link);

	wlr_cursor_attach_input_device(server->cursor->wlr_cursor, device);
}

void wc_pointers_init(struct wc_server *server) {
	wl_list_init(&server->pointers);
}

void wc_pointers_fini(struct wc_server *server) {
	struct wc_pointer *pointer;
	struct wc_pointer *temp;
	wl_list_for_each_safe(pointer, temp, &server->pointers, link) {
		wc_pointer_removed(&pointer->destroy, NULL);
	}
}
