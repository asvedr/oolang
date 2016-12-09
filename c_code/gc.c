#include <stdlib.h>
#include <stdio.h>
#include "gc.h"

#define DEF_SOFT_INC       3//10
#define DEF_MAX_AGE        2//5
#define DEF_HARD_INC       3//10
#define DEF_HARD_CALL_FREQ 5//100

/*
	- auto call soft GC when softSpace is full and user call 'newObject'
	- auto call hard GC when softCall used N times(hardCallFreq)
	- 
*/

typedef struct {
	GCObject *var;
	char      age;
} GCVar;

static GCVar     *softSpace;
static int        softVarCount;
static int        softSpaceLen; 
static GCObject **hardSpace;
static int        hardVarCount;
static int        hardSpaceLen;

static int        softCalls; // var for autocall hardGC

Var VNULL;

void initGC() {
	NEWINT(VNULL, 0);
	softSpace    = malloc(sizeof(GCVar) * DEF_SOFT_INC);
	softSpaceLen = DEF_SOFT_INC;
	softVarCount = 0;
	hardSpace    = malloc(sizeof(GCObject**) * DEF_HARD_INC);
	hardSpaceLen = DEF_HARD_INC;
	hardVarCount = 0;
	softCalls    = 0;
}

void callGCHard();

void callGCSoft() {
	// GOING FROM END TO START.
	// BECAUSE OF LESS COUNT OF USING SHIFT IF USE THIS ORDER
	#define REMOVE_FROM_SPACE \
		if(i+1 < softVarCount) {\
			/* SHIFT */\
			for(int j=i+1; j<softVarCount; ++j)\
				softSpace[j-1] = softSpace[j];\
		}\
		softVarCount -= 1;
	for(int i = softVarCount - 1; i >= 0; --i) {
		if(softSpace[i].var -> refCount < 1) {
			// OBJECT CAN BE REMOVED
			GCObject* var = softSpace[i].var;
			if(var -> destructor != NULL)
				var -> destructor(var -> data);
			free(var);
			REMOVE_FROM_SPACE
		} else {
			// OBJECT MUST BE SAVED
			softSpace[i].age += 1;
			if(softSpace[i].age >= DEF_MAX_AGE) {
				// MUST MOVE TO HARD
				// CHANGING HARD SIZE
				if(hardVarCount + 1 == hardSpaceLen) {
					hardSpaceLen += DEF_HARD_INC;
					hardSpace = realloc(hardSpace, sizeof(GCObject**) * hardSpaceLen);
				}
				// MOVING TO HARD
				hardSpace[hardVarCount] = softSpace[i].var;
				hardVarCount += 1;
				REMOVE_FROM_SPACE
			}
			// ELSE NOTHING CHANGING
		}
	}
	#undef REMOVE_FROM_SPACE
	// CUTTING SEQ IF NEED
	if(softSpaceLen - softVarCount > DEF_SOFT_INC * 2) {
		softSpaceLen = softVarCount + DEF_SOFT_INC;
		softSpace = realloc(softSpace, sizeof(GCVar) * softSpaceLen);
	}
	softCalls = (softCalls + 1) % DEF_HARD_CALL_FREQ;
	if(softCalls == 0)
		callGCHard();
}

void callGCHard() {
	for(int i = hardVarCount - 1; i >= 0; --i) {
		if(hardSpace[i] -> refCount < 1) {
			// REMOVE OBJECT
			if(hardSpace[i] -> destructor != NULL) {
				hardSpace[i] -> destructor(hardSpace[i]);
			}
			free(hardSpace[i]);
			// REMOVE POINTER FROM SPACE
			if(i+1 < hardVarCount) {
				// SHIFT
				for(int j = i + 1; j < hardVarCount; ++j)
					hardSpace[j-1] = hardSpace[j];
			}
			// DEC SEQ
			hardVarCount -= 1;
		} else {
			// SAVE OBJECT. DOING NOTHING
		}
	}
	// CUTTING SEQ IF NEED
	if(hardSpaceLen - hardVarCount > DEF_HARD_INC * 2) {
		hardSpaceLen = hardVarCount + DEF_HARD_INC;
		hardSpace = realloc(hardSpace, sizeof(GCObject**) * hardSpaceLen);
	}
}

// for manual using
void callGCFull() {
	softCalls = -1;
	callGCSoft();
}

GCObject* newVarGC(Destructor dest) {
	if(softVarCount + 1 >= softSpaceLen) {
		// NEED GC
		callGCSoft();
	}
	if(softVarCount + 1 >= softSpaceLen) {
		// NEED GROW SOFT SPACE
		softSpaceLen = softVarCount + DEF_SOFT_INC;
		softSpace = realloc(softSpace, sizeof(GCVar) * softSpaceLen);
	}
	GCObject *var = malloc(sizeof(GCObject));
	var -> destructor = dest;
	softSpace[softVarCount].var = var;
	softSpace[softVarCount].age = 0;
	return var;
}

void statusGC(int *scount, int *hcount, int *slen, int *hlen) {
	*scount = softVarCount;
	*hcount = hardVarCount;
	*slen = softSpaceLen;
	*hlen = hardSpaceLen;
}
