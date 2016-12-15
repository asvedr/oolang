#include <stdio.h>
#include "gc.h"
#include "vector.h"
#include "vstr.h"

/*
void showStat(){
	int cs, ch, ls, lh;
	statusGC(&cs, &ch, &ls, &lh);
	printf("count s: %d, count h: %d, len s: %d, len h: %d\n", cs, ch, ls, lh);
}

int main() {
	#ifdef DEBUG
	printf("DEBUG MODE\n");
	#else 
	printf("NO DEBUG MODE\n");
	#endif
	initGC();
	Var strA = strFromCStr("A str");
	Var strB = strFromCStr("B str");
	INCLINK(strA);
	INCLINK(strB);
	Var vlen;
	NEWINT(vlen, 0);
	Var vec = vectorNew(vlen);
	INCLINK(vec);
	DECLINK(strA);
	DECLINK(strB);
	vectorPush(vec, strA);
	vectorPush(vec, strB);

	callGCSoft();

	Var vec2 = vectorNew(vlen);
	vectorPush(vec, vec2);
	Var strOut = strFromCStr("#str out");
	vectorPush(vec2, strOut);

	strPrint(strA);
	strPrint(strB);
	strPrint(strOut);

	callGCSoft();

	vectorPop(vec);

	//callGCSoft();
	callGCFull();

	callGCFull();

	showStat();
}
*/
