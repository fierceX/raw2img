CC=clang
CFLAGS=-O3 -I. -w -march=native

CFLAGS+=-DUSE_ZLIB -DUSE_JPEG -DUSE_JPEG8
CFLAGS+=-I/opt/homebrew/opt/jpeg/include/
CFLAGS+=-I/opt/homebrew/opt/libraw/include/

LDADD+=-L/opt/homebrew/opt/libraw/lib -L/opt/homebrew/opt/jpeg/lib -L/opt/homebrew/opt/little-cms2/lib -lraw -llcms2 -ljpeg -lz

GIT_SHA1=""
CSTFLAGS=$(CFLAGS) -DLIBRAW_NOTHREADS

bin: lut3d.c raw2jpg.c main.c
	${CC} -DLIBRAW_NOTHREADS  ${CFLAGS} -o raw2jpg main.c lut3d.c raw2jpg.c -lm -lstdc++ ${LDADD} -DGIT_SHA1=\"${GIT_SHA1}\"

# libraw2jpg.dylib: lut3d.c raw2jpg.c
# 	${CC} -DLIBRAW_NOTHREADS  ${CFLAGS} -shared -o libraw2jpg.dylib lut3d.c raw2jpg.c -lm -lstdc++ ${LDADD} -DGIT_SHA1=\"${GIT_SHA1}\"
