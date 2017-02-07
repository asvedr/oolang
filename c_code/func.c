#include <stdlib.h>
#include "func.h"

unsigned int _reg_err_key;
Var _reg_result;
Var _reg_exc_val;

void initFRegs() {
	_reg_err_key = 0;
	NEWINT(_reg_result, 0);
	NEWINT(_reg_exc_val, 0);
}

static void destructorClos(void *link) {
	Closure* clos = (Closure*)link;
	for(int i=0; i<clos -> envSize; ++i) {
		DECLINK(clos -> env[i]);
	}
	free(clos -> env);
	free(clos);
}

static void destructorMeth(void *link) {
	Closure* clos = (Closure*)link;
	DECLINK(clos -> env[0]);
	free(clos -> env);
	free(clos);
}

//void destructorFun(void *clos) {
//	free(clos);
//}

Var newFunc(void *func) {
	Var obj;
	Closure* clos = malloc(sizeof(Closure));
//	NEWOBJ(obj, clos, destructorFun);
	NEWOBJ(obj, clos, free);
	clos -> func = func;
	return obj;
//	clos -> env = NULL;
//	clos -> envSize = 0;
}

Var newClosure(int envSize, void *func, Closure **out) {
	Var obj;
	Closure* clos = malloc(sizeof(Closure));
	NEWOBJ(obj, clos, destructorClos);
	clos -> func = func;
	clos -> env = malloc(sizeof(Var) * envSize);
	clos -> envSize = envSize;
	*out = clos;
	return obj;
}

Var methodClosure(Var *self, void *func) {
	Var obj;
	Closure* clos = malloc(sizeof(Closure));
	NEWOBJ(obj, clos, destructorMeth);
	clos -> env = malloc(sizeof(Var));
	*(clos -> env) = *self;
	clos -> func = func;
	return obj;
}
