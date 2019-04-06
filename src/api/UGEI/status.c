/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "cJSON/cJSON.h"
#include "engine.h"
#include "utils/utils.h"

void
engine_ugei_call_status(struct Engine *engine, cJSON *response)
{
	char *mode;
	switch (engine->mode) {
		case MODE_IDLE:
			mode = "idle";
		case MODE_SEARCH:
			mode = "searching";
		case MODE_EXIT:
			mode = "exiting";
		default:
			mode = "?";
	}
	cJSON *result = cJSON_AddObjectToObject(response, "result");
	cJSON_AddStringToObject(result, "mode", mode);
}