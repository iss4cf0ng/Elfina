CC = gcc
CFLAGS = -Wall -Wextra -O2 -g
TARGET = elf_runner
SRCS = ain.c elf_loader.c
OBJS = $(SRCS:.c=.o)

.PHONY: all clean test

all: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(CFLAGS) -o $@ $^

%.o: %.c
	$(CC) $(CFLAGS) -c -o $@ $<

#Build a simple static test binary to run
test_hello: test_hello.c
	$(CC) -static -o $@ $<

#Full demo: build runner, build test binary, run it through the loader
test: $(TARGET) test_hello
	@echo "--- Running test_hello via elf_runner ---"
	./$(TARGET) ./test_hello Alice Bob

clean:
	rm -f $(OBJS) $(TARGET) test_hello