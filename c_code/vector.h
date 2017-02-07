#ifndef VECTOR_H
#define VECTOR_H

#include "gc.h"
#include "func.h"

typedef struct {
	Var* data;
	int size;
} Vector;

void vectorNew(Var*,Var);
void vectorResize(Var*,/*Var,*/Var);
void vectorLen(Var* /*,Var*/);
void vectorPush(Var* /*,Var*/,Var);
void vectorPop(Var* /*,Var*/);
void vectorGet(Var* /*,Var*/,Var); 
void vectorPut(Var* /*,Var*/,Var,Var);

#endif
