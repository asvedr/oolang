#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include "debug.h"
#include "vstr.h"

static void destructor(void *data) {
	Str *s = data;
	static char buf[256];
	for(int i=0; i<s->size; ++i)
		buf[i] = s -> data[i];
	buf[s -> size] = '\0';
	debugLog("dest call for %s\n", buf);
	//printf("dest call for %s\n", buf);
	free(s -> data);
	free(s);
}

void strNew(Var* _ , Var vsize) {
	Str *str = malloc(sizeof(Str));
	int size = VINT(vsize);
	str -> size = size;
	if(size == 0)
		str -> data = NULL;
	else
		str -> data = malloc(sizeof(char) * size);
	for(int i=0; i<size; ++i)
		str -> data[i] = '\0';
	Var var;
	NEWOBJ(var, (void*)str, destructor);
	RETURN(var);
}

void strFromRaw(Var* _ , char* seq, int size) {
	Str *str = malloc(sizeof(Str));
	str -> size = size;
	if(size == 0)
		str -> data = NULL;
	else
		str -> data = malloc(sizeof(char) * size);
	for(int i=0; i<size; ++i)
		str -> data[i] = seq[i];
	Var var;
	NEWOBJ(var, (void*)str, destructor);
	RETURN(var);
}

void strFromCStr(Var* _, char* seq) {
	int size = strlen(seq);
	Str *str = malloc(sizeof(Str));
	str -> size = size;
	if(size == 0)
		str -> data = NULL;
	else
		str -> data = malloc(sizeof(char) * size);
	for(int i=0; i<size; ++i)
		str -> data[i] = seq[i];
	Var var;
	NEWOBJ(var, (void*)str, destructor);
	RETURN(var);
}

#ifdef DEBUG
void strPrint(Var s) {
	Str *str = (Str*)VAL(s);
	static char out[256];
	int i=0;
	for(; i<str -> size; ++i) {
		out[i] = str -> data[i];
	}
	out[i] = '\0';
	printf(">> %s\n", out);
}
#endif

void strLen(Var* self /*, Var v*/) {
	Var out;
	NEWINT(out, ((Str*)VAL(*self)) -> size);
	RETURN(out);
}

void strResize(Var* self , /*Var s,*/Var sz) {
	Str* str = (Str*)VAL(*self);
	int size = VINT(sz);
	if(size < 0)
		THROW(INDEXERR);
	if(str -> size < size) {
		// JUST ADD NULLS
		str -> data = realloc(str -> data, sizeof(char) * size);
		for(int i=str->size; i<size; ++i)
			str -> data[i] = '\0';
	} else {
		// RESIZE
		str -> data = realloc(str -> data, sizeof(Var) * size);
	}
	RETURNNULL;
}

void strGet(Var* self , /*Var s,*/ Var ind) {
	// TODO: CHECK OUT THROW
	Str* str = (Str*)VAL(*self);
	int index = VINT(ind);
	if(index < 0 || index >= str -> size)
		THROW( INDEXERR);
	Var out;
	NEWINT(out, str -> data[index]);
	RETURN( out);
}

void strPut(Var* self , /*Var s,*/ Var ind, Var val) {
	// TODO: CHECK OUT THROW
	Str* str = (Str*)VAL(*self);
	int index = VINT(ind);
	if(index < 0 || index >= str -> size)
		THROW( INDEXERR);
	str -> data[index] = (char)VINT(val);
	RETURNNULL
}

void strSub(Var* self , /*Var s,*/ Var vfrom, Var vto) {
	// TODO: CHECK OUT THROW
	Str* str = (Str*)VAL(*self);
	int from = VINT(vfrom);
	int to = VINT(vto);
	int size = to - from;
	if(from < 0 || from >= str -> size || to < from || to > str -> size)
		THROW( INDEXERR);
	Str* out = malloc(sizeof(Str));
	out -> size = size;
	if(size == 0)
		out -> data = NULL;
	else
		out -> data = malloc(sizeof(char) * size);
	for(int i=0; i<size; ++i)
		out -> data[i] = str -> data[from + i];
	
	Var outv;
	NEWOBJ(outv, out, destructor);
	RETURN( outv);
}

void strConc(Var* self , /*Var a,*/ Var b) {
	CHECKNULL( b);
	Str* str = (Str*)VAL(*self);
	Str* add = (Str*)VAL(b);
	int pt = str -> size;
	str -> data = realloc(str -> data, sizeof(char) * (str -> size + add -> size));
	str -> size += add -> size;
	for(int i=0; i<add -> size; ++i) {
		str -> data[pt + i] = add -> data[i];
	}
	RETURNNULL
}
