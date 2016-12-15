#ifndef CLASS_H
#define CLASS_H

#include "gc.h"

typedef struct {
	void *virtuals; // links to pure funcs
	Var  *props;
	int  propCnt;
} Object;

#define GETVIRT(var,ind) ((Object*)VAL(var)) -> virtuals[ind]
#define GETPROP(var,ind) ((Object*)VAL(var)) -> props[ind]

Var newObjectNV(int); // no virtuals
Var newObject(int,int);

#endif
