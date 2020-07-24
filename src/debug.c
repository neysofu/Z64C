#include "switches.h"
#include <stdarg.h>
#include <stdio.h>

void
debug_printf(const char *format, ...)
{
#if DEBUG_MESSAGES
	va_list args;
	va_start(args, format);
	fprintf(stdout, "[DEBUG] ");
	vfprintf(stdout, format, args);
	va_end(args);
#endif
}