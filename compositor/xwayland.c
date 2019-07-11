#include "xwayland.h"

#include <stdlib.h>

#include <wlr/util/log.h>

#include "server.h"
#include "view.h"

void wc_xwayland_surface_destroy(struct wl_listener *listener, void *data) {
	struct wc_view *view = wl_container_of(listener, view, destroy);

	wl_list_remove(&view->link);

	wl_list_remove(&view->map.link);
	wl_list_remove(&view->unmap.link);
	wl_list_remove(&view->configure.link);
	wl_list_remove(&view->destroy.link);

	free(view);
}

static void wc_xwayland_request_configure(
		struct wl_listener *listener, void *data) {
	struct wc_view *view = wl_container_of(listener, view, configure);

	struct wlr_xwayland_surface_configure_event *event = data;

	view->geo.x = event->x;
	view->geo.y = event->y;
	view->geo.width = event->width;
	view->geo.height = event->height;

	wlr_xwayland_surface_configure(view->xwayland_surface, event->x, event->y,
			event->width, event->height);
}

static void wc_xwayland_commit(struct wl_listener *listener, void *data) {
	struct wc_view *view = wl_container_of(listener, view, commit);
	if (!view->mapped) {
		return;
	}

	struct wlr_xwayland_surface *xwayland_surface = view->xwayland_surface;
	pixman_region32_t damage;
	pixman_region32_init(&damage);
	wlr_surface_get_effective_damage(xwayland_surface->surface, &damage);
	wc_view_damage(view, &damage);

	struct wlr_box size = {.x = xwayland_surface->x,
			.y = xwayland_surface->y,
			.width = xwayland_surface->width,
			.height = xwayland_surface->height};

	bool size_changed =
			view->geo.width != xwayland_surface->surface->current.width ||
			view->geo.height != xwayland_surface->surface->current.height;

	if (size_changed) {
		wc_view_damage_whole(view);
		view->geo.width = xwayland_surface->surface->current.width;
		view->geo.height = xwayland_surface->surface->current.height;
		wc_view_damage_whole(view);
	}

	if (view->pending_geometry.x != view->geo.x) {
		view->geo.x = view->pending_geometry.x + view->pending_geometry.width -
				size.width;
	}
	if (view->pending_geometry.y != view->geo.y) {
		view->geo.y = view->pending_geometry.y + view->pending_geometry.height -
				size.height;
	}

	wc_view_damage_whole(view);

	view->pending_serial = 0;
	view->is_pending_serial = false;

	pixman_region32_fini(&damage);
}

static void wc_xwayland_surface_map(struct wl_listener *listener, void *data) {
	struct wc_view *view = wl_container_of(listener, view, map);
	struct wlr_xwayland_surface *surface = data;

	view->mapped = true;
	wc_focus_view(view);

	view->geo.x = surface->x;
	view->geo.y = surface->y;
	view->geo.width = surface->width;
	view->geo.height = surface->height;

	view->commit.notify = wc_xwayland_commit;
	wl_signal_add(
			&view->xwayland_surface->surface->events.commit, &view->commit);

	wc_view_damage_whole(view);
}

static void wc_xwayland_surface_unmap(
		struct wl_listener *listener, void *data) {
	struct wc_view *view = wl_container_of(listener, view, unmap);
	view->mapped = false;

	wl_list_remove(&view->commit.link);

	wc_view_damage_whole(view);
}

static void wc_xwayland_new_surface(struct wl_listener *listener, void *data) {
	struct wc_server *server =
			wl_container_of(listener, server, new_xwayland_surface);
	struct wlr_xwayland_surface *xwayland_surface = data;

	struct wc_view *view = calloc(1, sizeof(struct wc_view));
	view->server = server;
	view->xwayland_surface = xwayland_surface;
	view->surface_type = WC_XWAYLAND;

	view->map.notify = wc_xwayland_surface_map;
	view->unmap.notify = wc_xwayland_surface_unmap;
	view->configure.notify = wc_xwayland_request_configure;
	view->destroy.notify = wc_xwayland_surface_destroy;

	wl_signal_add(&xwayland_surface->events.map, &view->map);
	wl_signal_add(&xwayland_surface->events.unmap, &view->unmap);
	wl_signal_add(
			&xwayland_surface->events.request_configure, &view->configure);
	wl_signal_add(&xwayland_surface->events.destroy, &view->destroy);

	wl_list_insert(&server->views, &view->link);
}

void wc_xwayland_init(struct wc_server *server) {
	server->xwayland =
			wlr_xwayland_create(server->wl_display, server->compositor, false);

	server->new_xwayland_surface.notify = wc_xwayland_new_surface;
	wl_signal_add(&server->xwayland->events.new_surface,
			&server->new_xwayland_surface);

	if (server->xwayland == NULL) {
		abort();
	}
}

void wc_xwayland_fini(struct wc_server *server) {
	wlr_xwayland_destroy(server->xwayland);
	server->xwayland = NULL;

	wl_list_remove(&server->new_xwayland_surface.link);
}
