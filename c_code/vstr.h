typedef struct {
	char* raw;
	int size;
} Str;

Var strNew(Var);
Var strFromRaw(char*,int);
void strPrint(Var);
Var strLen(Var);
Var strResize(Var,Var);
Var strGet(Var,Var); 
Var strPut(Var,Var,Var);
Var strSub(Var,Var,Var);
Var strConc(Var,Var);
