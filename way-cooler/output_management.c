#include "output_management.h"

#include <wlr/types/wlr_output_management_v1.h>

#include "server.h"

static void output_apply(struct wl_listener *listener, void *data) {
	struct wc_server *server =
			wl_container_of(listener, server, output_manager_apply);
	struct wlr_output_configuration_v1 *config = data;

	wlr_output_configuration_v1_destroy(config);
}

static void output_test(struct wl_listener *listener, void *data) {
	struct wlr_output_configuration_v1 *config = data;

	// TODO: Implement test-only mode
	wlr_output_configuration_v1_send_succeeded(config);
	wlr_output_configuration_v1_destroy(config);
}

void wc_output_management_init(struct wc_server *server) {
	server->output_manager = wlr_output_manager_v1_create(server->wl_display);

	server->output_manager_apply.notify = output_apply;
	wl_signal_add(&server->output_manager->events.apply,
			&server->output_manager_apply);

	server->output_manager_test.notify = output_test;
	wl_signal_add(
			&server->output_manager->events.test, &server->output_manager_test);
}
