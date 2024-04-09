CC=clang
CFLAGS=-O3 -I. -w -march=native
CFLAGS+=-static

CFLAGS+=-DUSE_ZLIB -DUSE_JPEG -DUSE_JPEG8
CFLAGS+=-I/usr/local/include/

LDADD+=-L/usr/local/lib -lraw -llcms2 -ljpeg -lz

GIT_SHA1=""
CSTFLAGS=$(CFLAGS) -DLIBRAW_NOTHREADS

bin: main.c lut3d.c
	${CC} -DLIBRAW_NOTHREADS  ${CFLAGS} -o raw2jpg main.c lut3d.c -lm -lstdc++ ${LDADD} -DGIT_SHA1=\"${GIT_SHA1}\"