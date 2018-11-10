/**
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * @file engine.h
 * @brief The core components of Z64C.
 * This file contains the API to the very core of Z64C.
 */

#pragma once

#include "chess/board.h"
#include "chess/legality.h"
#include "chess/result.h"
#include "chess/termination.h"
#include "game.h"
#include "mode.h"
#include "search/scontroller.h"
#include "search/ttable.h"
#include "settings.h"
#include <stdbool.h>

#define BUFFER_SIZE 1024

/// @brief A self-contained chess engine instance.
/// The @c Engine structure contains all the necessary components to make Z64C
/// work.
/// This structure doesn't takes up very little memory by itself. The most
/// resource-hungry component by far and large is 'struct TTable'.
struct Engine
{
	struct Board board;
	struct Result result;
	struct Clock *time_control[2];
	struct Settings settings;
	enum Mode mode;
	int8_t exit_status;
	/** The heavy lifts. */
	struct TTable *ttable;
	struct SController *scontroller;
	struct Network *network;
};

/// @brief Instantiates a new chess engine.
///
/// The newly created chess engine is
/// still asleep, without responding to commands.
///
/// @see engine_main
struct Engine *
engine_new(void);

/// @brief Gracefully kill a struct Engine instance.
///
/// @param engine A pointer to the engine instance to kill.
void
engine_free(struct Engine *engine);

/// @brief Initializes @p engine and starts listening from a ZUCI shell.
///
/// Once initialized, @p param will be actively listening on `stdin` for ZUCI
/// commands. Output information will be written to `stdout` and error messages
/// on `stderr`. This function will return only upon failure or a \a quit
/// command.
///
/// @param engine The engine instance to initialize.
/// @return A non-zero error code on failure, zero on success.
int8_t
engine_main(struct Engine *engine);

/**
 * @brief Run a Remote Procedure Call (RPC) and return the engine's response.
 * @param A pointer to the request string.
 * @return A pointer to the response string. The pointer will be NULL on exit
 * and the exit status will be stored in 'engine->exit_status'.
 */
const char *
engine_rpc(struct Engine *engine, const char *cmd);
