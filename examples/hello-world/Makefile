.PHONY: run clean

ANNABELLA_PREFIX ?= ~/.local
ANNABELLA ?= ${ANNABELLA_PREFIX}/bin/annabella
ANNABELLA_INCLUDE ?= ${ANNABELLA_PREFIX}/include
ANNABELLA_LIB ?= ${ANNABELLA_PREFIX}/lib

BIN := hello-world

OBJECTS := $(wildcard *.adb)
OBJECTS := ${OBJECTS:%.adb=%.o}

OBJECTS += fake-std.o

run: ${BIN}
	LD_LIBRARY_PATH=${ANNABELLA_LIB} ./${BIN}

${BIN}: ${OBJECTS} ${ANNABELLA_LIB}/libannabella-rt.so
	gcc -o $@ -L ${ANNABELLA_LIB} ${OBJECTS} -l annabella-rt

%.o: %.c ~/.local/include/annabella-rt.h
	gcc -I ${ANNABELLA_INCLUDE} -c $<

%.c: %.adb ${ANNABELLA}
	${ANNABELLA} $< > $@

clean:
	rm -fv *.o ${BIN}
