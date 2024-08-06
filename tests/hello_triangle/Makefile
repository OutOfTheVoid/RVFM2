SELF_DIR := $(dir $(lastword $(MAKEFILE_LIST)))

OBJCOPY = riscv64-unknown-elf-objcopy
MKDIR = mkdir
RMDIR = rm -rf
COPY  = cp
OBJDUMP = riscv64-unknown-elf-objdump
SHASM = $(SELF_DIR)../../target/debug/shader_assembler

OBJCOPY_FLAGS = --output-target=binary

SRC_DIR = $(SELF_DIR)src
TARGET_DIR = $(SELF_DIR)bin
CARGO_TARGET_DIR = $(SELF_DIR)target/riscv32i-unknown-none-elf

LD_SCRIPT = ../../sdk/link/link.ld

TARGET_NAME = hello_triangle
TARGET_ELF = $(TARGET_NAME).bin.elf
TARGET = $(TARGET_NAME).bin

CARGO = cargo

default: build disassemble

$(CARGO_TARGET_DIR)/release/$(TARGET_NAME): $(TARGET_DIR)/vshader.bin $(TARGET_DIR)/fshader.bin
	$(CARGO) build --release

$(TARGET_DIR):
	$(MKDIR) $(TARGET_DIR)

$(TARGET_DIR)/$(TARGET_ELF):
	$(COPY) $(CARGO_TARGET_DIR)/release/$(TARGET_NAME) $(TARGET_DIR)/$(TARGET_ELF)

$(TARGET_DIR)/$(TARGET): $(CARGO_TARGET_DIR)/release/$(TARGET_NAME) $(TARGET_DIR)
	$(OBJCOPY) $(CARGO_TARGET_DIR)/release/$(TARGET_NAME) $(OBJCOPY_FLAGS) $(TARGET_DIR)/$(TARGET)

$(TARGET_DIR)/vshader.bin: $(SRC_DIR)/shader.shasm $(TARGET_DIR) $(SHASM)
	$(SHASM) $(SRC_DIR)/shader.shasm -v $(TARGET_DIR)/vshader.bin

$(TARGET_DIR)/fshader.bin: $(SRC_DIR)/shader.shasm $(TARGET_DIR) $(SHASM)
	$(SHASM) $(SRC_DIR)/shader.shasm -f $(TARGET_DIR)/fshader.bin

$(SHASM):
	$(CARGO) -Z unstable-options -C ../.. build --bin shader_assembler

disassembly.txt: $(CARGO_TARGET_DIR)/release/$(TARGET_NAME)
	$(OBJDUMP) $(CARGO_TARGET_DIR)/release/$(TARGET_NAME) --disassemble-all > disassembly.txt

build: $(TARGET_DIR)/$(TARGET) $(TARGET_DIR)/$(TARGET_ELF) 

disassemble: disassembly.txt

clean:
	$(CARGO) clean
	$(RMDIR) bin




