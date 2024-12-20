TARGET := hello

CC := riscv64-linux-musl-gcc
STRIP := riscv64-linux-musl-strip


CC := docker run --platform linux/riscv64 -v `pwd`:/tmp -w /tmp myrisc/alpine:3 gcc
STRIP := docker run --platform linux/riscv64 -v `pwd`:/tmp -w /tmp myrisc/alpine:3 strip


# 编译
# gcc -nostdlib  -static -o hello_c.bin hello_c/hello.c  -L. -l:libaxlibc.a -lc -emain

all: $(TARGET)

%: %.c
# docker run --platform linux/riscv64 -v `pwd`:/tmp -w /tmp myrisc/alpine:3 gcc --static  $<  -o  $@
# docker run --platform linux/riscv64 -v `pwd`:/tmp -w /tmp myrisc/alpine:3 strip  $@
# docker run --platform linux/riscv64 -it drujensen/riscv-ubuntu 
	$(CC) -static $< -o $@
	$(STRIP) $@

clean:
	@rm -rf ./$(TARGET)
