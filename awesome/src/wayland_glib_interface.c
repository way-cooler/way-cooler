#include <wayland-client-core.h>
#include <glib.h>

void refresh_awesome(void);

/* Instance of an event source that we use to integrate the wayland event queue
 * with GLib's MainLoop.
 */
struct InterfaceEventSource {
	GSource source;
	struct wl_display *display;
	struct wl_event_queue *queue;
	gpointer fd_tag;
};

/* This function is called to prepare polling event source. We just flush
 * and indicate that we have no timeouts, nor are currently pending.
 */
static gboolean interface_prepare(GSource *base, gint *timeout)
{
	struct InterfaceEventSource *interface_source
		= (struct InterfaceEventSource *) base;

	wl_display_flush(interface_source->display);
	*timeout = -1;

	return FALSE;
}

/* This function is called after file descriptors were checked. We indicate that
 * we need to be dispatched if any events on the epoll fd we got from libwayland
 * are pending / need to be handled.
 */
static gboolean interface_check(GSource *base)
{
	struct InterfaceEventSource *interface_source
		= (struct InterfaceEventSource *) base;
	GIOCondition condition
		= g_source_query_unix_fd(base, interface_source->fd_tag);

	/* We need to dispatch if anything happened on the fd */
	return condition != 0;
}

/* This function is called to actually "do" some work. We just run the wayland
 * event queue with a timeout of 0.
 */
static gboolean interface_dispatch(GSource *base, GSourceFunc callback,
		gpointer data)
{
	struct InterfaceEventSource *interface_source
		= (struct InterfaceEventSource *) base;
	wl_display_roundtrip(interface_source->display);

	refresh_awesome();

	(void) callback;
	(void) data;

	return G_SOURCE_CONTINUE;
}

static GSourceFuncs interface_funcs = {
	.prepare  = interface_prepare,
	.check    = interface_check,
	.dispatch = interface_dispatch,
};

/* Initialise and register an event source with GLib. This event source
 * integrates the wayland event queue with the GLib main loop.
 */
void wayland_glib_interface_init(struct wl_display *display,
		struct wl_event_queue *queue)
{
	struct InterfaceEventSource *interface_source;
	GSource *source = g_source_new(&interface_funcs, sizeof(*interface_source));

	interface_source = (struct InterfaceEventSource *) source;
	interface_source->display = display;
	interface_source->queue = queue;
	wl_display_roundtrip(interface_source->display);

	interface_source->fd_tag =
		g_source_add_unix_fd(source, wl_display_get_fd(display),
			G_IO_IN | G_IO_ERR | G_IO_HUP);
	g_source_set_can_recurse(source, TRUE);

	g_source_attach(source, NULL);
}
