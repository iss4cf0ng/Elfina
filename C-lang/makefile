# Elfina
#
# Layout:
#   src/            C and asm source files + headers
#   bin/            all compiled outputs (created automatically)
#   bin/obj64/      64-bit intermediate object files
#   bin/obj32/      32-bit intermediate object files
#
# Usage:
#   make            – compile bin/elfina (x86-64) and bin/elfina32 (x86)
#   make test       – build then run all native tests
#   make cross      – build cross-arch test ELFs -> bin/
#   make test-cross – probe cross-arch ELFs with --info
#   make clean      – remove bin/ entirely
#
# Requirements for 32-bit build:
#   apt install gcc-multilib

CC     = gcc
CFLAGS = -Wall -Wextra -O2 -g
 
SRC_DIR = src
BIN_DIR = bin
OBJ64   = $(BIN_DIR)/obj64
OBJ32   = $(BIN_DIR)/obj32
 
SRCS = $(SRC_DIR)/main.c $(SRC_DIR)/elf_loader.c
 
# x86-64: each .c in src/ -> .o in bin/obj64/
OBJS64   = $(patsubst $(SRC_DIR)/%.c, $(OBJ64)/%.o, $(SRCS))
TARGET64 = $(BIN_DIR)/elfina
 
# x86 (32-bit): each .c in src/ -> .o in bin/obj32/
OBJS32   = $(patsubst $(SRC_DIR)/%.c, $(OBJ32)/%.o, $(SRCS))
TARGET32 = $(BIN_DIR)/elfina32

# Host arch detection
ARCH := $(shell uname -m)
ifeq ($(ARCH),x86_64)
  STATIC_FLAGS = -static
  PIE_FLAGS    = -pie -fPIE
  HAS_M32      = 1
endif
ifeq ($(ARCH),aarch64)
  STATIC_FLAGS = -static
  PIE_FLAGS    = -pie -fPIE
  HAS_M32      = 0
endif
ifeq ($(ARCH),armv7l)
  STATIC_FLAGS = -static
  PIE_FLAGS    = -pie -fPIE
  HAS_M32      = 0
endif

# Cross-compiler toolchain names
CROSS_ARM64 = aarch64-linux-gnu-gcc
CROSS_ARM32 = arm-linux-gnueabihf-gcc
CROSS_X86   = i686-linux-gnu-gcc
CROSS_RV64  = riscv64-linux-gnu-gcc
 
.PHONY: all test cross test-cross clean

# Default: build both elfina (64) and elfina32 (32) if possible
ifeq ($(HAS_M32),1)
all: $(TARGET64) $(TARGET32) \
     $(BIN_DIR)/test_static \
     $(BIN_DIR)/test_dynamic \
     $(BIN_DIR)/test_asm
else
all: $(TARGET64) \
     $(BIN_DIR)/test_static \
     $(BIN_DIR)/test_dynamic \
     $(BIN_DIR)/test_asm
	@echo "NOTE: bin/elfina32 skipped (requires x86-64 host + gcc-multilib)"
endif

# Create output directories
$(BIN_DIR):
	mkdir -p $@
 
$(OBJ64): | $(BIN_DIR)
	mkdir -p $@
 
$(OBJ32): | $(BIN_DIR)
	mkdir -p $@
 
# Compile: x86-64 objects  (src/foo.c -> bin/obj64/foo.o)
$(OBJ64)/%.o: $(SRC_DIR)/%.c | $(OBJ64)
	$(CC) $(CFLAGS) -m64 -I$(SRC_DIR) -c -o $@ $<
 
# Compile: x86 objects (src/foo.c -> bin/obj32/foo.o)
# -m32 generates 32-bit code; requires gcc-multilib on Debian/Ubuntu
$(OBJ32)/%.o: $(SRC_DIR)/%.c | $(OBJ32)
	$(CC) $(CFLAGS) -m32 -I$(SRC_DIR) -c -o $@ $<

# Static link: bin/elfina (x86-64)
$(TARGET64): $(OBJS64) | $(BIN_DIR)
	$(CC) $(CFLAGS) -m64 -static -o $@ $^
	@echo "Built x86-64 : $@"
 
# Static link: bin/elfina32 (x86)
$(TARGET32): $(OBJS32) | $(BIN_DIR)
	$(CC) $(CFLAGS) -m32 -static -o $@ $^
	@echo "Built x86    : $@"
 
# Native test
$(BIN_DIR)/test_static: $(SRC_DIR)/test_hello.c | $(BIN_DIR)
	$(CC) $(STATIC_FLAGS) -o $@ $<
 
$(BIN_DIR)/test_dynamic: $(SRC_DIR)/test_hello.c | $(BIN_DIR)
	$(CC) -o $@ $<
 
$(BIN_DIR)/test_asm: $(SRC_DIR)/test_asm.S | $(BIN_DIR)
	$(CC) -nostdlib $(STATIC_FLAGS) -o $@ $<
 
# Test
test: all
	@echo ""
	@echo "=== [64-bit] Info probe: static binary ==="
	$(TARGET64) --info $(BIN_DIR)/test_static
	@echo ""
	@echo "=== [64-bit] Info probe: dynamic binary ==="
	$(TARGET64) --info $(BIN_DIR)/test_dynamic
	@echo ""
	@echo "=== [64-bit] Run: dynamic via memfd ==="
	$(TARGET64) --memfd $(BIN_DIR)/test_dynamic Alice Bob
	@echo ""
	@echo "=== [64-bit] Run: static via memfd ==="
	$(TARGET64) --memfd $(BIN_DIR)/test_static Alice Bob
	@echo ""
	@echo "=== [64-bit] Run: raw asm via mmap loader ==="
	$(TARGET64) --mmap $(BIN_DIR)/test_asm
ifeq ($(HAS_M32),1)
	@echo ""
	@echo "=== [32-bit] Info probe: static binary ==="
	$(TARGET32) --info $(BIN_DIR)/test_static
	@echo ""
	@echo "=== [32-bit] Run: static via memfd ==="
	$(TARGET32) --memfd $(BIN_DIR)/test_static Alice Bob
endif

# Cross-compiled test binaries
cross: $(BIN_DIR)/test_arm64 \
       $(BIN_DIR)/test_arm32 \
       $(BIN_DIR)/test_x86   \
       $(BIN_DIR)/test_riscv64
 
$(BIN_DIR)/test_arm64: $(SRC_DIR)/test_hello.c | $(BIN_DIR)
	$(CROSS_ARM64) -static -o $@ $< || echo "SKIP: $(CROSS_ARM64) not found"
 
$(BIN_DIR)/test_arm32: $(SRC_DIR)/test_hello.c | $(BIN_DIR)
	$(CROSS_ARM32) -static -o $@ $< || echo "SKIP: $(CROSS_ARM32) not found"
 
$(BIN_DIR)/test_x86: $(SRC_DIR)/test_hello.c | $(BIN_DIR)
	$(CROSS_X86) -static -o $@ $< || echo "SKIP: $(CROSS_X86) not found"
 
$(BIN_DIR)/test_riscv64: $(SRC_DIR)/test_hello.c | $(BIN_DIR)
	$(CROSS_RV64) -static -o $@ $< || echo "SKIP: $(CROSS_RV64) not found"
 
test-cross: $(TARGET64) cross
	@for f in test_arm64 test_arm32 test_x86 test_riscv64; do \
	  [ -f "$(BIN_DIR)/$$f" ] && \
	  echo "" && echo "=== $$f ===" && \
	  $(TARGET64) --info $(BIN_DIR)/$$f || true; \
	done

# Clean: wipe bin/ entirely (obj64/, obj32/, and all binaries)
clean:
	rm -rf $(BIN_DIR)