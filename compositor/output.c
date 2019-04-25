#define _POSIX_C_SOURCE 200809L

#include "output.h"

#include <stdlib.h>
#include <time.h>
#include <assert.h>

#include <wayland-server.h>
#include <wlr/types/wlr_matrix.h>
#include <wlr/types/wlr_output.h>
#include <wlr/types/wlr_output_layout.h>
#include <wlr/render/wlr_renderer.h>

#include "view.h"
#include "server.h"

/* Used to move all of the data necessary to render a surface from the top-level
 * frame handler to the per-surface render function. */
struct render_data {
	struct wlr_output *output;
	struct wlr_renderer *renderer;
	struct wc_view *view;
	struct timespec *when;
};

static void render_surface(struct wlr_surface* surface,
		int sx, int sy, void *data);

static void wc_new_output(struct wl_listener* listener, void* data) {
	struct wc_server* server = wl_container_of(listener, server, new_output);
	struct wlr_output* output = data;

	if (!wl_list_empty(&output->modes)) {
		struct wlr_output_mode* mode =
			wl_container_of(output->modes.prev, mode, link);
		wlr_output_set_mode(output, mode);
	}

	struct wc_output* wc_output = calloc(1, sizeof(struct wc_output));
	wc_output->output = output;
	wc_output->server = server;
	wc_output->frame.notify = wc_output_frame;
	wl_signal_add(&output->events.frame, &wc_output->frame);
	wl_list_insert(&server->outputs, &wc_output->link);

	wlr_output_layout_add_auto(server->output_layout, output);
	wlr_output_create_global(output);
}

void wc_output_frame(struct wl_listener* listener, void* data) {
	struct wc_output* output = wl_container_of(listener, output, frame);
	struct wlr_output* wlr_output = output->output;
	struct wlr_renderer* renderer = wlr_backend_get_renderer(wlr_output->backend);
	assert(renderer);

	struct timespec now;
	clock_gettime(CLOCK_MONOTONIC, &now);
	//TODO wlr_output_attach_render(wlr_output, NULL);
	if (!wlr_output_make_current(wlr_output, NULL)) {
		return;
	}

	int width, height;
	wlr_output_effective_resolution(wlr_output, &width, &height);
	wlr_renderer_begin(renderer, width, height);

	float color[4] = { 0.25f, 0.25f, 0.25f, 1 };
	wlr_renderer_clear(renderer, color);

	struct wc_view* view;
	wl_list_for_each_reverse(view, &output->server->views, link) {
		if (!view->mapped) {
			continue;
		}
		struct render_data rdata = {
			.output = output->output,
			.view = view,
			.renderer = renderer,
			.when = &now
		};

		wlr_xdg_surface_for_each_surface(view->xdg_surface,
				render_surface, &rdata);
	}

	wlr_output_render_software_cursors(wlr_output, NULL);

	//TODO use wlr_output_commit(wlr_output);
	wlr_renderer_end(renderer);
	wlr_output_swap_buffers(wlr_output, NULL, NULL);
}


static void render_surface(struct wlr_surface* surface,
		int sx, int sy, void *data) {
	struct render_data* rdata = data;
	struct wc_view* view = rdata->view;
	struct wlr_output* output = rdata->output;

	struct wlr_texture* texture = wlr_surface_get_texture(surface);
	if (texture == NULL) {
		return;
	}

	double ox = 0, oy = 0;
	wlr_output_layout_output_coords(
		view->server->output_layout, output, &ox, &oy);
	ox += view->x + sx, oy += view->y + sy;

	struct wlr_box box = {
		.x = ox * output->scale,
		.y = oy * output->scale,
		.width = surface->current.width * output->scale,
		.height = surface->current.height * output->scale,
	};
	float matrix[9];
	enum wl_output_transform transform =
		wlr_output_transform_invert(surface->current.transform);
	wlr_matrix_project_box(matrix, &box, transform, 0,
						   output->transform_matrix);

	wlr_render_texture_with_matrix(rdata->renderer, texture, matrix, 1);

	wlr_surface_send_frame_done(surface, rdata->when);
}

void init_output(struct wc_server* server) {
	server->output_layout = wlr_output_layout_create();
	wl_list_init(&server->outputs);
	server->new_output.notify = wc_new_output;
	wl_signal_add(&server->backend->events.new_output, &server->new_output);
}
