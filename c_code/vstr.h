#ifndef STR_H
#define STR_H

#include "gc.h"
#include "func.h"

typedef struct {
	char* data;
	int size;
} Str;

void strNew(Var*,Var);
void strFromRaw(Var*,char*,int);
void strFromCStr(Var*,char*);
#ifdef DEBUG
void strPrint(Var);
#endif
void strLen(Var*);
void strResize(Var*,Var);
void strGet(Var*,Var); 
void strPut(Var*,Var,Var);
void strSub(Var*,Var,Var);
void strConc(Var*,Var);

#endif
