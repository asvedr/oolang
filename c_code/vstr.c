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

Var strNew(Var vsize) {
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
	return var;
}

Var strFromRaw(char* seq, int size) {
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
	return var;
}

Var strFromCStr(char* seq) {
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
	return var;
}

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

Var strLen(Var v) {
	Var out;
	NEWINT(out, ((Str*)VAL(v)) -> size);
	return out;
}

Var strResize(Var s,Var sz) {
	Str* str = (Str*)VAL(s);
	int size = VINT(sz);
	if(str -> size < size) {
		// JUST ADD NULLS
		str -> data = realloc(str -> data, sizeof(char) * size);
		for(int i=str->size; i<size; ++i)
			str -> data[i] = '\0';
	} else {
		// RESIZE
		str -> data = realloc(str -> data, sizeof(Var) * size);
	}
	VNULL;
}

Var strGet(Var s, Var ind) {
	// TODO: CHECK OUT THROW
	Str* str = (Str*)VAL(s);
	Var out;
	NEWINT(out, str -> data[VINT(ind)]);
	return out;
}

Var strPut(Var s, Var ind, Var val) {
	// TODO: CHECK OUT THROW
	Str* str = (Str*)VAL(s);
	str -> data[VINT(ind)] = (char)VINT(val);
	VNULL;
}

Var strSub(Var s, Var vfrom, Var vto) {
	// TODO: CHECK OUT THROW
	Str* str = (Str*)VAL(s);
	int from = VINT(vfrom);
	int to = VINT(vto);
	int size = to - from;
	
	Str* res = malloc(sizeof(Str));
	res -> size = size;
	if(size == 0)
		res -> data = NULL;
	else
		res -> data = malloc(sizeof(char) * size);
	for(int i=0; i<size; ++i)
		res -> data[i] = str -> data[from + i];
	
	Var out;
	NEWOBJ(out, res, destructor);
	return out;
}

Var strConc(Var a, Var b) {
	Str* str = (Str*)VAL(a);
	Str* add = (Str*)VAL(b);
	int pt = str -> size;
	str -> data = realloc(str -> data, sizeof(char) * (str -> size + add -> size));
	str -> size += add -> size;
	for(int i=0; i<add -> size; ++i) {
		str -> data[pt + i] = add -> data[i];
	}
	VNULL;
}
