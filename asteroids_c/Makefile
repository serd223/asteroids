WIN_LIB_ARGS = -Lexternal/raylib/lib -lraylib -lwinmm -lopengl32 -lgdi32 -luser32 -lkernel32 -Xlinker /NODEFAULTLIB -lmsvcrt -lucrt -lmsvcprt -lvcruntime -lshell32
LINUX_LIB_ARGS = -Lexternal/raylib/lib -l:libraylib.a -lm
COMPILE_FLAGS = -Iexternal/raylib/include/ -std=c99 -Wall -pedantic

comma := ,
empty :=
space := $(empty) $(empty)
FLAGS_LIST = $(subst $(space),$(comma),$(COMPILE_FLAGS))
a := $(file > .clangd, CompileFlags:)
b := $(file >> .clangd, 	Add: [$(FLAGS_LIST)])

win: build/main.exe
linux: build/main

build/main.exe: main.c
	clang main.c -o build/main.exe $(COMPILE_FLAGS) $(WIN_LIB_ARGS)

build/main: main.c
	clang main.c -o build/main $(COMPILE_FLAGS) $(LINUX_LIB_ARGS)
