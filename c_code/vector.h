#ifndef VECTOR_H
#define VECTOR_H

#include "gc.h"
#include "func.h"

typedef struct {
	Var* data;
	int size;
} Vector;

void vectorNew(Var*,FunRes*,Var);
void vectorResize(Var*,FunRes*,/*Var,*/Var);
void vectorLen(Var*,FunRes* /*,Var*/);
void vectorPush(Var*,FunRes* /*,Var*/,Var);
void vectorPop(Var*,FunRes* /*,Var*/);
void vectorGet(Var*,FunRes* /*,Var*/,Var); 
void vectorPut(Var*,FunRes* /*,Var*/,Var,Var);

#endif
