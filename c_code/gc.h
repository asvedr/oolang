#ifndef GC_H
#define GC_H

#ifdef __cplusplus
extern "C" {
#endif

typedef void (*Destructor)(void*);

typedef struct {
	int         refCount;
	void       *data;
	Destructor  destructor;
} GCObject;

typedef struct {
	char          obj;
	union {
		GCObject *link;
		void     *plink;
		int       inum;
		double    fnum;
	};
} Var;

#define INCLINK(var) {\
	if((var).obj) {\
		(var).link -> refCount += 1;\
	}\
}
#define DECLINK(var) {\
	if((var).obj) {\
		(var).link -> refCount -= 1;\
	}\
}
#define ASSIGN(var,val) {\
	DECLINK((var));\
	INCLINK((val));\
	(var) = (val);\
}

#define VAL(v) ((v).link -> data)
#define VINT(v) (v.inum)
#define VREAL(v) (v.fnum)
#define PLINK(v) (v.plink)

GCObject* newVarGC(Destructor);

#define NEWOBJ(var,val,destr) {\
	var.obj = 1;\
	var.link = newVarGC(destr);\
	var.link -> data = val;\
}
#define NEWINT(var,n) {\
	var.obj = 0;\
	var.inum = n;\
}
#define NEWREAL(var,n) {\
	var.obj = 0;\
	var.fnum = n;\
}
#define NEWPRIMLINK(var, l) {\
	var.obj = 0;\
	var.plink = l;\
}

//extern Var VNULL;
#define VNULL {\
	Var out;\
	out.obj = 0;\
	out.inum = 0;\
	return out;\
}

void initGC();
void callGCSoft();
void callGCFull();
// out params: count in soft, count in hard, len of soft, len of hard
void statusGC(int*, int*, int*, int*);
// args: max soft space len, max mid space len, hard call freq
//void comfigureGC(int, int, );

#ifdef __cplusplus
}
#endif

#endif
