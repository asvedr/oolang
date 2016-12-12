CC=gcc -O0 -g 

all: clear gc.o vector.o vstr.o test

clear:
	rm *.o || true
	rm test || true

test:
	$(CC) -otest test.c vector.o vstr.o gc.o
