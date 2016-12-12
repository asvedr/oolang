#ifndef VECTOR_H
#define VECTOR_H

#include "gc.h"

typedef struct {
	Var* data;
	int size;
} Vector;

Var vectorNew(Var);
Var vectorResize(Var,Var);
Var vectorLen(Var);
Var vectorPush(Var,Var);
Var vectorPop(Var);
Var vectorGet(Var,Var); 
Var vectorPut(Var,Var,Var); 

#endif
