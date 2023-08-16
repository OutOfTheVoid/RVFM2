
OBJCOPY = riscv64-unknown-elf-objcopy
MKDIR = mkdir
RMDIR = rm -rf
OBJDUMP = riscv64-unknown-elf-objdump

CARGO = cargo

OBJCOPY_FLAGS = --output-target=binary

TARGET_DIR = bin

LD_SCRIPT = ../../sdk/link/link.ld

TARGET_NAME = template
TARGET = $(TARGET_NAME).bin

default: build disassemble

target/riscv32i-unknown-none-elf/release/$(TARGET_NAME):
	$(CARGO) build --release

$(TARGET_DIR):
	$(MKDIR) $(TARGET_DIR)

$(TARGET_DIR)/$(TARGET): target/riscv32i-unknown-none-elf/release/$(TARGET_NAME) $(TARGET_DIR)
	$(OBJCOPY) target/riscv32i-unknown-none-elf/release/$(TARGET_NAME) $(OBJCOPY_FLAGS) $(TARGET_DIR)/$(TARGET)

disassembly.txt: target/riscv32i-unknown-none-elf/release/$(TARGET_NAME)
	$(OBJDUMP) target/riscv32i-unknown-none-elf/release/$(TARGET_NAME) --disassemble-all > disassembly.txt

build: $(TARGET_DIR)/$(TARGET)

disassemble: disassembly.txt

clean:
	$(CARGO) clean
	$(RMDIR) bin




