#include <stdlib.h>
#include "class.h"

void destructorNV(void *link) {
	Object* obj = (Object*)link;
	for(int i=0; i<obj -> propCnt; ++i) {
		DECLINK(obj -> props[i]);
	}
	free(obj -> props);
}

void destructor(void *link) {
	Object* obj = (Object*)link;
	for(int i=0; i<obj -> propCnt; ++i) {
		DECLINK(obj -> props[i]);
	}
	free(obj -> props);
	free(obj -> virtuals);
}

Var newObjectNV(int len) {
	Var res;
	Object* obj = malloc(sizeof(Object));
	obj -> props = calloc(sizeof(Var), len);
	NEWOBJ(res, obj, destructorNV);
	return res;
}

Var newObject(int vlen, int plen) {
	Var res;
	Object* obj = malloc(sizeof(Object));
	obj -> props = calloc(sizeof(Var), plen);
	obj -> virtuals = malloc(sizeof(void*) * vlen);
	NEWOBJ(res, obj, destructor);
	return res;
}
