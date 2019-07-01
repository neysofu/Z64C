/* Copyright (c) Filippo Costa - All Rights Reserved
 * Unauthorized copying of this file, via any medium is strictly prohibited
 * PROPRIETARY AND CONFIDENTIAL */

#include "chess/coordinates.h"
#include "chess/bb.h"
#include "chess/color.h"
#include <assert.h>

char
rank_to_char(Rank r)
{
	return r + '1';
}

char
file_to_char(File f)
{
	return f + 'a';
}

File
char_to_file(char c)
{
	return (c >= 'a' && c <= 'h') ? c - 'a' : FILE_NONE;
}

Rank
char_to_rank(char c)
{
	return (c >= '1' && c <= '8') ? c - '1' : RANK_NONE;
}

File
square_file(Square square)
{
	return square >> 3;
}

Rank
square_rank(Square square)
{
	return square & 0x7;
}

Bitboard
square_to_bb(Square square)
{
	return 1ULL << square;
}
Square
bb_to_square(Bitboard bb)
{
	assert(bb);
	return MSB(bb);
}

Bitboard
file_to_bb(File f)
{
	return 0xff << (f << FILES_COUNT);
}

Bitboard
rank_to_bb(Rank r)
{
	return 0x101010101010101ULL << r;
}

Square
square_new(File file, Rank rank)
{
	return file << 3 | rank;
}

Square
lsb_to_square(int lsb)
{
	return lsb;
}

Square
msb_to_square(int msb)
{
	return SQUARE_MAX - msb;
}
