# Elfina
#
# Layout:
#   src/	C and asm source files + headers
#   bin/	all compiled outputs (created automatically)
#
# Usage:
#   make				– compile all sources into bin/elfina
#   make test			– build then run all native tests
#   make cross			– build cross-arch test ELFs -> bin/
#   make test-cross		– probe cross-arch ELFs with --info
#   make clean			– remove bin/ entirely

CC     = gcc
CFLAGS = -Wall -Wextra -O2 -g

SRC_DIR = src
BIN_DIR = bin

SRCS   = $(SRC_DIR)/main.c $(SRC_DIR)/elf_loader.c
OBJS   = $(patsubst $(SRC_DIR)/%.c, $(BIN_DIR)/%.o, $(SRCS))
TARGET = $(BIN_DIR)/elfina

ARCH := $(shell uname -m)
ifeq ($(ARCH),x86_64)
  STATIC_FLAGS = -static
  PIE_FLAGS    = -pie -fPIE
endif
ifeq ($(ARCH),aarch64)
  STATIC_FLAGS = -static
  PIE_FLAGS    = -pie -fPIE
endif
ifeq ($(ARCH),armv7l)
  STATIC_FLAGS = -static
  PIE_FLAGS    = -pie -fPIE
endif

CROSS_ARM64 = aarch64-linux-gnu-gcc
CROSS_ARM32 = arm-linux-gnueabihf-gcc
CROSS_X86   = i686-linux-gnu-gcc
CROSS_RV64  = riscv64-linux-gnu-gcc

.PHONY: all test cross test-cross clean

all: $(TARGET) \
     $(BIN_DIR)/test_static \
     $(BIN_DIR)/test_dynamic \
     $(BIN_DIR)/test_asm

$(BIN_DIR):
	mkdir -p $@

$(BIN_DIR)/%.o: $(SRC_DIR)/%.c | $(BIN_DIR)
	$(CC) $(CFLAGS) -I$(SRC_DIR) -c -o $@ $<

$(TARGET): $(OBJS) | $(BIN_DIR)
	$(CC) $(CFLAGS) -o $@ $^

$(BIN_DIR)/test_static: $(SRC_DIR)/test_hello.c | $(BIN_DIR)
	$(CC) $(STATIC_FLAGS) -o $@ $<

$(BIN_DIR)/test_dynamic: $(SRC_DIR)/test_hello.c | $(BIN_DIR)
	$(CC) -o $@ $<

$(BIN_DIR)/test_asm: $(SRC_DIR)/test_asm.S | $(BIN_DIR)
	$(CC) -nostdlib $(STATIC_FLAGS) -o $@ $<

test: all
	@echo ""
	@echo "=== Info probe: static binary ==="
	$(TARGET) --info $(BIN_DIR)/test_static
	@echo ""
	@echo "=== Info probe: dynamic binary ==="
	$(TARGET) --info $(BIN_DIR)/test_dynamic
	@echo ""
	@echo "=== Run: dynamic via memfd ==="
	$(TARGET) --memfd $(BIN_DIR)/test_dynamic Alice Bob
	@echo ""
	@echo "=== Run: static via memfd ==="
	$(TARGET) --memfd $(BIN_DIR)/test_static Alice Bob
	@echo ""
	@echo "=== Run: raw asm via mmap loader ==="
	$(TARGET) --mmap $(BIN_DIR)/test_asm

cross: $(BIN_DIR)/test_arm64 \
       $(BIN_DIR)/test_arm32 \
       $(BIN_DIR)/test_x86   \
       $(BIN_DIR)/test_riscv64

$(BIN_DIR)/test_arm64: $(SRC_DIR)/test_hello.c | $(BIN_DIR)
	$(CROSS_ARM64) -static -o $@ $< || echo "SKIP: $(CROSS_ARM64) not found"

$(BIN_DIR)/test_arm32: $(SRC_DIR)/test_hello.c | $(BIN_DIR)
	$(CROSS_ARM32) -static -o $@ $< || echo "SKIP: $(CROSS_ARM32) not found"

$(BIN_DIR)/test_x86: $(SRC_DIR)/test_hello.c | $(BIN_DIR)
	$(CROSS_X86)   -static -o $@ $< || echo "SKIP: $(CROSS_X86) not found"

$(BIN_DIR)/test_riscv64: $(SRC_DIR)/test_hello.c | $(BIN_DIR)
	$(CROSS_RV64)  -static -o $@ $< || echo "SKIP: $(CROSS_RV64) not found"

test-cross: $(TARGET) cross
	@for f in test_arm64 test_arm32 test_x86 test_riscv64; do \
	  [ -f "$(BIN_DIR)/$$f" ] && \
	  echo "" && echo "=== $$f ===" && \
	  $(TARGET) --info $(BIN_DIR)/$$f || true; \
	done

clean:
	rm -rf $(BIN_DIR)