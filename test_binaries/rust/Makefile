SELF_DIR := $(dir $(lastword $(MAKEFILE_LIST)))

OBJCOPY = riscv64-unknown-elf-objcopy
MKDIR = mkdir
RMDIR = rm -rf
OBJDUMP = riscv64-unknown-elf-objdump

OBJCOPY_FLAGS = --output-target=binary

TARGET_DIR = $(SELF_DIR)bin
CARGO_TARGET_DIR = $(SELF_DIR)target/riscv32i-unknown-none-elf

LD_SCRIPT = ../../sdk/link/link.ld

TARGET_NAME = main
TARGET = $(TARGET_NAME).bin

CARGO = cargo

default: build disassemble

$(CARGO_TARGET_DIR)/release/$(TARGET_NAME):
	$(CARGO) build --release

$(TARGET_DIR):
	$(MKDIR) $(TARGET_DIR)

$(TARGET_DIR)/$(TARGET): $(CARGO_TARGET_DIR)/release/$(TARGET_NAME) $(TARGET_DIR)
	$(OBJCOPY) $(CARGO_TARGET_DIR)/release/$(TARGET_NAME) $(OBJCOPY_FLAGS) $(TARGET_DIR)/$(TARGET)

disassembly.txt: $(CARGO_TARGET_DIR)/release/$(TARGET_NAME)
	$(OBJDUMP) $(CARGO_TARGET_DIR)/release/$(TARGET_NAME) --disassemble-all > disassembly.txt

build: $(TARGET_DIR)/$(TARGET)

disassemble: disassembly.txt

clean:
	$(CARGO) clean
	$(RMDIR) bin




