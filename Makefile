CFLAGS=-O3 -I. -w -march=native
CC=clang
CXX=g++

CFLAGS+=-DUSE_ZLIB
LDADD+=-lz

CFLAGS+=-I/usr/local/include/
LDADD+=-L/usr/local/lib -static -lraw

LDADD+=-L/usr/local/lib -static -llcms2

CFLAGS+=-DUSE_JPEG -I/usr/local/include
LDADD+=-L/usr/local/lib -static -ljpeg 
CFLAGS+=-DUSE_JPEG8

CSTFLAGS=$(CFLAGS) -DLIBRAW_NOTHREADS

bin: main.c lut3d.c
	${CC} -DLIBRAW_NOTHREADS  ${CFLAGS} -o raw2jpg main.c lut3d.c -lm -lstdc++ ${LDADD}