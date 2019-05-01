#define _POSIX_C_SOURCE 200809L

#include <getopt.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <wordexp.h>
#include <unistd.h>

#include <wlr/backend.h>
#include <wlr/util/log.h>

#include "server.h"

const char* WC_HELP_MESSAGE =
	"Usage: %s [OPTION] startup_command\n"
	"\n"
	"  -c <command>           Execute the command after startup.\n"
	"  -h                     Show help message and quit.\n"
	"  -d                     Turn on debugging"
	"\n";

const char* WC_GETOPT_OPTIONS =
#ifdef __GNUC__
"+"
#endif
"hc:d";

const char* WC_BINARY_PATH = NULL;


void print_usage(void) {
	printf(WC_HELP_MESSAGE, WC_BINARY_PATH);
}

int main(int argc, char* argv[]) {
	WC_BINARY_PATH = argv[0];
	wlr_log_init(WLR_DEBUG, NULL);
	char* startup_cmd = NULL;

	int c;
	while ((c = getopt(argc, argv, WC_GETOPT_OPTIONS)) != -1) {
		switch (c) {
		case 'd':
			WC_DEBUG = 1;
			break;
		case 'c':
			startup_cmd = strdup(optarg);
			break;
		case 'h':
		default:
			print_usage();
			exit(1);
		}
	}
	if (optind < argc) {
		print_usage();
		exit(1);
	}

	struct wc_server server = {0};
	if (!init_server(&server)) {
		wlr_log(WLR_ERROR, "Could not initialize Wayland resources");
		exit(1);
	}
	wlr_log(WLR_INFO, "Running Wayland compositor on WAYLAND_DISPLAY=%s",
			server.wayland_socket);
	if (!wlr_backend_start(server.backend)) {
		wlr_backend_destroy(server.backend);
		wl_display_destroy(server.wl_display);
		return false;
	}
	setenv("WAYLAND_DISPLAY", server.wayland_socket, true);

	if (startup_cmd) {
		wlr_log(WLR_INFO, "Executing \"%s\"", startup_cmd);
		if (fork() == 0) {
			execl("/bin/sh", "/bin/sh", "-c", startup_cmd, (void*)NULL);
		}
	}

	wl_display_run(server.wl_display);
	fini_server(&server);
	free(startup_cmd);
}
