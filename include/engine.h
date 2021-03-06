/* SPDX-License-Identifier: GPL-3.0-only */

#ifndef ZULOID_ENGINE_H
#define ZULOID_ENGINE_H

#include "cache/cache.h"
#include "chess/position.h"
#include "chess/termination.h"
#include "eval.h"
#include "time/game_clock.h"
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

#define ENGINE_LOGF(engine, ...)                                                           \
	engine_logf(engine, __FILE__, __LINE__, __func__, __VA_ARGS__)

enum Status
{
	// No background activity. The engine is just waiting for user input.
	STATUS_IDLE,
	// While still accepting user input, the engine is searching the game tree
	// in the background.
	STATUS_SEARCH,
	// The engine will not accept other user input. This mode is not
	// recoverable.
	STATUS_EXIT,
};

struct Engine;

struct Config
{
	bool debug;
	float move_selection_noise;
	// Must be in the range [0,1]. It measures the engine's sense of
	// superiority and thus reluctancy to draw. When set to 0, draws are
	// effectively treated as wins; when set to 1, as losses. Tune it according
	// the opponent's rating for best performance.
	float contempt;
	// Must be in the range [0,1]. The greater its value, the more selective
	// the engine will be in searching the game tree. Warning! Even little
	// adjustments can have extensive influence over the gameplay. 0.5 is the
	// most performant option. */
	float selectivity;
	bool ponder;
	size_t max_nodes_count;
	size_t max_depth;
	FILE *output;
	void (*protocol)(struct Engine *, const char *);
};

struct Engine
{
	// Only one position at the time.
	struct Board board;
	struct Cache *cache;
	struct Agent *agent;
	struct Eval eval;
	struct Tablebase *tablebase;
	int32_t seed;
	// A straightforward activity indicator. Both `main` and engine commands
	// might want to know if the engine is doing background computation or
	// what.
	enum Status status;
	/* -- Search limits. */
	struct TimeControl *time_controls[2];
	struct GameClock game_clocks[2];
	struct Config config;
};

void
init_subsystems(void);

struct Engine *
engine_new(void);

void
engine_init(struct Engine *engine);

void
engine_call(struct Engine *engine, char *cmd);

void
engine_start_search(struct Engine *engine);
void
engine_stop_search(struct Engine *engine);

void
engine_logf(struct Engine *engine,
            const char *filename,
            size_t line_num,
            const char *function_name,
            ...);

void
engine_delete(struct Engine *engine);

#endif
