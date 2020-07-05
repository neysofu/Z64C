#include "engine.h"
#include "agent.h"
#include "cache/cache.h"
#include "chess/position.h"
#include "protocols.h"
#include <assert.h>
#include <stdarg.h>
#include <stdio.h>
#include <string.h>

struct Engine *
engine_new(void)
{
	struct Engine *engine = malloc(sizeof(struct Engine));
	if (engine) {
		*engine = (struct Engine){
			.position = POSITION_INIT,
			.time_controls = { NULL, NULL },
			.cache = NULL,
			.agent = agent_new(),
			.seed = 0xcfca130b,
#ifdef NDEBUG
			.debug = false,
#else
			.debug = true,
#endif
			.move_selection_noise = 0.005,
			.contempt = 0.65,
			.selectivity = 0.5,
			.protocol = PROTOCOL_UNKNOWN,
			.mode = MODE_IDLE,
			.exit_status = EXIT_SUCCESS,
		};
	}
	return engine;
}

int
engine_delete(struct Engine *engine)
{
	if (!engine) {
		return EXIT_SUCCESS;
	}
	int exit_status = engine->exit_status;
	time_control_delete(engine->time_controls[COLOR_WHITE]);
	time_control_delete(engine->time_controls[COLOR_BLACK]);
	cache_delete(engine->cache);
	agent_delete(engine->agent);
	free(engine);
	return exit_status;
}

void
engine_call(struct Engine *engine, char *cmd)
{
	assert(engine);
	assert(cmd);
	switch (engine->protocol) {
		case PROTOCOL_UNKNOWN:
			engine_unknown_protocol(engine, cmd);
			break;
		case PROTOCOL_UCI:
			engine_uci(engine, cmd);
			break;
		default:
			engine->mode = MODE_EXIT;
	}
}

void
engine_debugf(struct Engine *engine,
              const char *filename,
              size_t line_num,
              const char *function_name,
              ...)
{
	assert(engine);
	assert(filename);
	assert(function_name);
	if (!engine->debug) {
		return;
	}
	switch (engine->protocol) {
		case PROTOCOL_UCI:
			printf("info string ");
			break;
		case PROTOCOL_CECP:
			printf("# ");
			break;
		default:
			return;
	}
#ifndef NDEBUG
	printf("%s:%zu @ %s -- ", filename, line_num, function_name);
#endif
	va_list args;
	va_start(args, function_name);
	vprintf(va_arg(args, const char *), args);
	va_end(args);
}
