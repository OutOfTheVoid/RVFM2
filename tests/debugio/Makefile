AS = riscv64-unknown-elf-as
LD = riscv64-unknown-elf-ld
OBJCOPY = riscv64-unknown-elf-objcopy
MKDIR = mkdir
RMDIR = rm -rf

AS_FLAGS = -march=rv32imaf
LD_FLAGS = -T $(LD_SCRIPT) -nostdlib -b elf32-littleriscv
OBJCOPY_FLAGS = --output-target=binary

OBJ_DIR = obj
TARGET_DIR = bin
SRC_DIR = src

LD_SCRIPT = ../../sdk/link/link.ld

TARGET = debugio.bin

$(OBJ_DIR):
	$(MKDIR) $(OBJ_DIR)

$(TARGET_DIR):
	$(MKDIR) $(TARGET_DIR)

$(OBJ_DIR)/main.o: $(SRC_DIR) $(OBJ_DIR) $(SRC_DIR)/main.s
	$(AS) $(AS_FLAGS) $(SRC_DIR)/main.s -o $(OBJ_DIR)/main.o

$(TARGET_DIR)/$(TARGET).elf: $(TARGET_DIR) $(OBJ_DIR)/main.o
	$(LD) $(OBJ_DIR)/main.o -o $(TARGET_DIR)/$(TARGET).elf $(LD_FLAGS)

$(TARGET_DIR)/$(TARGET): $(TARGET_DIR)/$(TARGET).elf
	$(OBJCOPY) $(OBJCOPY_FLAGS) $(TARGET_DIR)/$(TARGET).elf $(TARGET_DIR)/$(TARGET)

all: $(TARGET_DIR)/$(TARGET)

clean:
	$(RMDIR) obj
	$(RMDIR) bin

