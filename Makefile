CFLAGS=-O3 -I. -w
CC=clang
CXX=g++

CFLAGS+=-DUSE_ZLIB
LDADD+=-lz

CFLAGS+=-I/usr/local/include/
LDADD+=-L/usr/local/lib -lraw

CFLAGS+=-DUSE_JPEG -I/usr/local/include
LDADD+=-L/usr/local/lib -ljpeg 
CFLAGS+=-DUSE_JPEG8

CSTFLAGS=$(CFLAGS) -DLIBRAW_NOTHREADS

bin: main.c
	${CC} -DLIBRAW_NOTHREADS  ${CFLAGS} -o raw2jpg main.c -lm -lstdc++ ${LDADD}