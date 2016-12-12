#ifndef DEBUG_H
#define DEBUG_H

#include <stdio.h>
#include <assert.h>

#ifdef DEBUG
	#define debugLog(...) {fprintf(stderr, __VA_ARGS__); fflush(stderr);}
#else
	#define debugLog(...) while(0);
#endif

#endif
