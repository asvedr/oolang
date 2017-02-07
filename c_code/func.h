#ifndef FUNC_H
#define FUNC_H

#include "gc.h"
#include "errors.h"
/*
typedef struct {
	Var           val;
	unsigned int  errKey; // 0: no err, >0: code of exception
} FunRes;


#define RETURN(ptr, _val) {\
	DECLINK(ptr -> val);\
	INCLINK(_val);\
	ptr -> val = _val;\
}
#define RETURNNULL(ptr) {return; }
#define THROWP(ptr, code, val) {ptr -> errKey = code; ASG(ptr -> val, val); return; }
#define THROW(ptr, code) {ptr -> errKey = code; return; }
#define NEWFRES(name) FunRes name; NEWINT(name.val, 0); name -> errKey = 0;
#define CHECKNULL(ptr, _val) if(!_val.obj) THROW(ptr, NULLPTRERR)
*/

extern unsigned int _reg_err_key;
extern Var _reg_result;
void initFRegs();
//extern int _i_result;
//extern double _r_result;

/*
	void funTemplate(args) {
		Var vars, ...;
		simpleCall (fres, args');
		
		exCall(fres, args')
		if(fres.errFlag)
			goto TRACE;

		return res;
		TRACE:
		DECLINK(vars, ...);
		*FunRes = *inheritFunRes;
	}
*/

//                   env,   res,  args ...
typedef void(*CFun0)(Var*,/*FunRes* */);
typedef void(*CFun1)(Var*,/*FunRes* */,Var);
typedef void(*CFun2)(Var*,/*FunRes* */,Var,Var);
typedef void(*CFun3)(Var*,/*FunRes* */,Var,Var,Var);
typedef void(*CFun4)(Var*,/*FunRes* */,Var,Var,Var,Var);
typedef void(*CFun5)(Var*,/*FunRes* */,Var,Var,Var,Var,Var);
typedef void(*CFunM)(Var*,/*FunRes* */,...);

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
