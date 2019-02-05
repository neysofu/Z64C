/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "chess/position.h"
#include <stdint.h>

struct Eval
{
	uint_fast32_t score;
	uint_fast32_t uncertainty;
};

void
positions_eval(const struct Position *position, struct Eval *eval)
{
}
