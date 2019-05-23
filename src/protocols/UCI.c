/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "cache/cache.h"
#include "chess/position.h"
#include "core/agent.h"
#include "core/search.h"
#include "engine.h"
#include "globals.h"
#include "utils.h"
#include "xxHash/xxhash.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

void
engine_uci_call_go(struct Engine *engine, char *cmd)
{
	assert(engine);
	char *token = strsep_whitespace(cmd);
	switch (XXH64(token, strlen(token), 0)) {
		case 0x2a8ef3657cf9a920: /* "wtime" */
			token = strsep_whitespace(cmd);
			engine->time_controls[COLOR_WHITE]->time_limit_in_seconds = atoi(token) * 1000;
			break;
		case 0xd3f6a6885c7c93a0: /* "btime" */
			token = strsep_whitespace(cmd);
			engine->time_controls[COLOR_BLACK]->time_limit_in_seconds = atoi(token) * 1000;
			break;
		case 0x71c2388517319e0c: /* "winc" */
			token = strsep_whitespace(cmd);
			engine->time_controls[COLOR_WHITE]->increment_in_seconds = atoi(token) * 1000;
			break;
		case 0xad513987341315ae: /* "binc" */
			token = strsep_whitespace(cmd);
			engine->time_controls[COLOR_BLACK]->increment_in_seconds = atoi(token) * 1000;
			break;
		case 0x18ebef875e97de86: /* "infinite" */
			time_control_delete(engine->time_controls[COLOR_WHITE]);
			time_control_delete(engine->time_controls[COLOR_BLACK]);
			break;
	}
	PUThread *thread = p_uthread_create((PUThreadFunc)(engine_search), &engine, false);
	printf("bestmove e2e4 ponder c7c5\n");
}

void
engine_uci_call_position(struct Engine *engine, char *cmd)
{
	assert(engine);
	assert(cmd);
	if (strcmp(strsep_whitespace(cmd), "startpos") == 0) {
		engine->position = POSITION_INIT;
	} else if (strcmp(strsep_whitespace(cmd), "fen") == 0) {
		position_init_from_fen(&engine->position, cmd);
	}
	while (strsep_whitespace(cmd)) {
		// TODO
		//	position_push(string_to_move(cmd_current(&cmd)));
	}
}

void
engine_uci_call_setoption(struct Engine *engine, char *cmd)
{
	assert(engine);
	assert(cmd);
	strsep_whitespace(cmd);
	uint64_t hash = 0;
	char *token = strsep_whitespace(cmd);
	while (strcmp(token, "value") != 0) {
		for (size_t i = 0; token[i]; i++) {
			token[i] = tolower(token[i]);
		}
		hash ^= XXH64(token, strlen(token), 0);
	}
	strsep_whitespace(cmd);
	switch (hash) {
		case 0xd8cdd8e8314c4147: /* "hash" */
			break;
		case 0xc057a61296095806: /* "nalimovpath" */
			break;
		case 0xcfaf77aca1f8feaa: /* "nalimovcache" */
			break;
		case 0x0a6f394a3987568a: /* "ponder" */
			break;
		case 0x487e0e93e2c2bb18: /* "ownbook" */
			break;
		case 0xc8922f8e470ffaee: /* "uci_showcurrline" */
			break;
		case 0xdf718624bc46e2ce: /* "uci_showrefutations" */
			break;
		case 0xf6253b92eb36d560: /* "uci_limitstrength" */
			break;
	}
}

void
engine_uci(struct Engine *engine, char *cmd)
{
	assert(engine);
	assert(cmd);
	char *token = strsep_whitespace(&cmd);
	switch (XXH64(token, strlen(token), 0)) {
		case 0x5000d8f2907d14e4: /* "d" */
			position_print(&engine->position);
			break;
		case 0xf80028c1113b2c9c: /* "uci" */
			printf("id name Z64C\n"
			       "id author Filippo Costa\n"
			       "option name Threads type spin default 1 min 1 max 512\n"
			       "option name Clear Hash type button\n"
			       "option name Hash type spin default 8 min 0 max 65536\n"
			       "option name Ponder type check default true\n"
			       "option name Skill Level type spin default 100 min 0 max 100\n"
			       "option name Move Overhead type spin default 30 min 0 max 5000\n"
			       "option name OwnBook\n"
			       "option name UCI_Opponent\n"
			       "option name UCI_Chess960 type check default false\n"
			       "option name UCI_AnalyseMode type check default false\n"
			       "option name UCI_EngineAbout type string default %s\n"
			       "option name SyzygyPath type string default <empty>\n"
			       "option name Style type combo default normal\n"
			       "uciok\n",
			       Z64C_COPYRIGHT);
			break;
		case 0x01fd51a2a6f9cc2f: /* "debug" */
			token = strsep_whitespace(&cmd);
			if (!token) {
				break;
			} else if (strcmp(token, "on") == 0) {
				engine->debug = true;
			} else if (strcmp(token, "off") == 0) {
				engine->debug = false;
			}
			break;
		case 0xd682e29388e0f4a3: /* "go" */
			engine_uci_call_go(engine, cmd);
			break;
		case 0x33e37def10e5d195: /* "isready" */
			printf("readyok\n");
			break;
		case 0x8a7ecc4137c6f2b0: /* "position" */
			engine_uci_call_position(engine, cmd);
			break;
		case 0x707db5f765aed6d8: /* "quit" */
			engine->mode = MODE_EXIT;
			break;
		case 0x26cc87cdbb3247ba: /* "setoption" */
			engine_uci_call_setoption(engine, cmd);
			break;
		default:
			printf("Unknown command: ", token);
	}
}