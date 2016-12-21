#ifndef STR_H
#define STR_H

#include "gc.h"
#include "func.h"

typedef struct {
	char* data;
	int size;
} Str;

void strNew(Var*,FunRes*,Var);
void strFromRaw(Var*,FunRes*,char*,int);
void strFromCStr(Var*,FunRes*,char*);
#ifdef DEBUG
void strPrint(Var);
#endif
void strLen(Var*,FunRes* /*,Var*/);
void strResize(Var*,FunRes* /*,Var*/,Var);
void strGet(Var*,FunRes* /*,Var*/,Var); 
void strPut(Var*,FunRes* /*,Var*/,Var,Var);
void strSub(Var*,FunRes* /*,Var*/,Var,Var);
void strConc(Var*,FunRes* /*,Var*/,Var);

#endif
