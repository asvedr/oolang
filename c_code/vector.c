#include <stdlib.h>
#include "debug.h"
#include "vector.h"

static void destructor(void *data) {
	Vector *vec = data;
	debugLog("dest call for vec len: %d\n", vec -> size);
	for(int i=0; i<vec -> size; ++i) {
		DECLINK(vec -> data[i]);
	}
	free(vec -> data);
	free(vec);
}

Var vectorNew(Var vsize) {
	Vector *vec = malloc(sizeof(Vector));
	int size = VINT(vsize);
	vec -> size = size;
	if(size == 0)
		vec -> data = NULL;
	else
		vec -> data = calloc(sizeof(Var), size);
//		vec -> data = malloc(sizeof(Var) * size);
//	for(int i=0; i<size; ++i)
//		NEWINT(vec -> data[i], 0);
	Var var;
	NEWOBJ(var, (void*)vec, destructor);
	return var;
}

Var vectorResize(Var v, Var sz) {
	Vector* vec = (Vector*)VAL(v);
	int size = VINT(sz);
	if(vec -> size < size) {
		// JUST ADD NULLS
		vec -> data = realloc(vec -> data, sizeof(Var) * size);
		for(int i=vec->size; i<size; ++i)
			NEWINT(vec -> data[i], 0);
	} else {
		// DEC LINKS AND RESIZE
		for(int i=vec -> size - 1; i >= size; --i)
			DECLINK(vec -> data[i]);
		vec -> data = realloc(vec -> data, sizeof(Var) * size);
	}
	VNULL;
}

Var vectorLen(Var v) {
	Var out;
	NEWINT(out, ((Vector*)VAL(v)) -> size);
	return out;
}

Var vectorPush(Var v, Var a) {
	Vector* vec = (Vector*)VAL(v);
	if(vec -> size > 0)
		vec -> data = realloc(vec -> data, sizeof(Var) * (vec -> size + 1));
	else
		vec -> data = malloc(sizeof(Var));
	vec -> data[vec -> size] = a;
	vec -> size += 1;
	INCLINK(a);
	VNULL;
}

Var vectorPop(Var v) {
	// TODO: CHECK OUT THROW
	Var out;
	Vector* vec = (Vector*)VAL(v);
	out = vec -> data[vec -> size - 1];
	vec -> size --;
	vec -> data = realloc(vec -> data, vec -> size);
	DECLINK(out);
	return out;
}

Var vectorGet(Var v, Var ind) {
	// TODO: CHECK OUT THROW
	Vector* vec = (Vector*)VAL(v);
	return vec -> data[VINT(ind)];
}

Var vectorPut(Var v, Var ind, Var val) {
	// TODO: CHECK OUT THROW
	Vector* vec = (Vector*)VAL(v);
	DECLINK(vec -> data[VINT(ind)]);
	INCLINK(val);
	vec -> data[VINT(ind)] = val;
	VNULL;
}
