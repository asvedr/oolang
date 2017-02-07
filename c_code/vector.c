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

void vectorNew(Var* _,  Var vsize) {
	Vector *vec = malloc(sizeof(Vector));
	int size = VINT(vsize);
	vec -> size = size;
	if(size == 0)
		vec -> data = NULL;
	else
		vec -> data = calloc(sizeof(Var), size);
	Var var;
	NEWOBJ(var, (void*)vec, destructor);
	RETURN( var);
}

void vectorResize(Var* self,  /*Var v,*/ Var sz) {
	Vector* vec = (Vector*)VAL(*self);
	int size = VINT(sz);
	if(size < 0)
		THROW( INDEXERR);
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
	RETURNNULL
}

void vectorLen(Var* self /*, Var v*/) {
	Var out;
	NEWINT(out, ((Vector*)VAL(*self)) -> size);
	RETURN(out);
}

void vectorPush(Var* self,  /*Var v,*/ Var a) {
	Vector* vec = (Vector*)VAL(*self);
	if(vec -> size > 0)
		vec -> data = realloc(vec -> data, sizeof(Var) * (vec -> size + 1));
	else
		vec -> data = malloc(sizeof(Var));
	vec -> data[vec -> size] = a;
	vec -> size += 1;
	INCLINK(a);
	RETURNNULL
}

void vectorPop(Var* self /*, Var v*/) {
	// TODO: CHECK OUT THROW: OK
	Var out;
	Vector* vec = (Vector*)VAL(*self);
	if(vec -> size == 0) {
		THROW( EMPTYVECERR);
	}
	out = vec -> data[vec -> size - 1];
	vec -> size --;
	vec -> data = realloc(vec -> data, vec -> size);
	DECLINK(out);
	RETURN( out);
}

void vectorGet(Var* self,  /*Var v,*/ Var ind) {
	// TODO: CHECK OUT THROW
	Vector* vec = (Vector*)VAL(*self);
	int index = VINT(ind);
	if(index < 0 || index >= vec -> size)
		THROW( INDEXERR)
	else
		RETURN( vec -> data[index]);
}

void vectorPut(Var* self,  /*Var v,*/ Var ind, Var val) {
	// TODO: CHECK OUT THROW
	Vector* vec = (Vector*)VAL(*self);
	int index = VINT(ind);
	if(index < 0 || index >= vec -> size)
		THROW(INDEXERR)
	else {
		ASSIGN(vec -> data[index], val);
		RETURNNULL
	}
}
