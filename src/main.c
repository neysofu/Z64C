/* SPDX-License-Identifier: GPL-3.0-only */

#include "engine.h"
#include "meta.h"
#include "feature_flags.h"
#include "utils.h"
#include <stdio.h>
#include <stdlib.h>

#ifdef ZULOID_ENABLE_SHOW_PID
#include <plibsys.h>
#endif

int
main(int argc, char **argv)
{
	init_subsystems();
	struct Engine *engine = engine_new();
	// The number sign ensures minimal possibility of accidental evaluation by
	// the client.
	printf("# Zuloid %s (%s)\n", ZULOID_VERSION_VERBOSE, ZULOID_BUILD_DATE);
	printf("# Copyright (c) 2018-2020 Filippo Costa\n");
#ifdef ZULOID_ENABLE_SHOW_PID
	printf("# Process ID: %d\n", p_process_get_current_pid());
#endif
	for (int i = 1; i < argc && engine->status != STATUS_EXIT; i++) {
		engine->config.protocol(engine, argv[i]);
	}
	while (engine->status != STATUS_EXIT) {
		char *line = read_line(stdin);
		engine->config.protocol(engine, line);
		free(line);
	}
	engine_delete(engine);
	p_libsys_shutdown();
	return EXIT_SUCCESS;
}
