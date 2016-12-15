#ifndef FUNC_H
#define FUNC_H

#include "gc.h"

typedef Var(*CFun0)(Var*);
typedef Var(*CFun1)(Var*,Var);
typedef Var(*CFun2)(Var*,Var,Var);
typedef Var(*CFun3)(Var*,Var,Var,Var);
typedef Var(*CFun4)(Var*,Var,Var,Var,Var);
typedef Var(*CFun5)(Var*,Var,Var,Var,Var,Var);
typedef Var(*CFunM)(Var*, ...);

typedef struct {
	Var  *env; // environment of func. array of vars. can be NULL
	int  envSize;
	void *func;
} Closure;

// macro for vars
#define CALL0(f)           ((CFun0)((Closure*)(f.link -> data)) -> func)( ((Closure*)(f.link -> data)) -> env )
#define CALL1(f,a)         ((CFun1)((Closure*)(f.link -> data)) -> func)( ((Closure*)(f.link -> data)) -> env, a )
#define CALL2(f,a,b)       ((CFun2)((Closure*)(f.link -> data)) -> func)( ((Closure*)(f.link -> data)) -> env, a, b )
#define CALL3(f,a,b,c)     ((CFun3)((Closure*)(f.link -> data)) -> func)( ((Closure*)(f.link -> data)) -> env, a, b, c )
#define CALL4(f,a,b,c,d)   ((CFun3)((Closure*)(f.link -> data)) -> func)( ((Closure*)(f.link -> data)) -> env, a, b, c, d )
#define CALL5(f,a,b,c,d,e) ((CFun3)((Closure*)(f.link -> data)) -> func)( ((Closure*)(f.link -> data)) -> env, a, b, c, d, e )
#define CALLM(f, ...)      ((CFun3)((Closure*)(f.link -> data)) -> func)( ((Closure*)(f.link -> data)) -> env, __VA_ARGS__ )

#define CALL0F(f)           ((CFun0)((Closure*)(f.link -> data)) -> func)( NULL )
#define CALL1F(f,a)         ((CFun1)((Closure*)(f.link -> data)) -> func)( NULL, a )
#define CALL2F(f,a,b)       ((CFun2)((Closure*)(f.link -> data)) -> func)( NULL, a, b )
#define CALL3F(f,a,b,c)     ((CFun3)((Closure*)(f.link -> data)) -> func)( NULL, a, b, c )
#define CALL4F(f,a,b,c,d)   ((CFun3)((Closure*)(f.link -> data)) -> func)( NULL, a, b, c, d )
#define CALL5F(f,a,b,c,d,e) ((CFun3)((Closure*)(f.link -> data)) -> func)( NULL, a, b, c, d, e )
#define CALLMF(f, ...)      ((CFun3)((Closure*)(f.link -> data)) -> func)( NULL, __VA_ARGS__ )

Var newFunc(void*);
Var newClosure(int,void*,Closure**);
Var methodClosure(Var*,void*);

#endif
