#ifndef STR_H
#define STR_H

#include "gc.h"

typedef struct {
	char* data;
	int size;
} Str;

Var strNew(Var);
Var strFromRaw(char*,int);
Var strFromCStr(char*);
void strPrint(Var);
Var strLen(Var);
Var strResize(Var,Var);
Var strGet(Var,Var); 
Var strPut(Var,Var,Var);
Var strSub(Var,Var,Var);
Var strConc(Var,Var);

#endif
