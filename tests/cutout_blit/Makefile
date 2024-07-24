AS = riscv64-unknown-elf-as
LD = riscv64-unknown-elf-ld
CC = riscv64-unknown-elf-gcc
OBJCOPY = riscv64-unknown-elf-objcopy
MKDIR = mkdir
RMDIR = rm -rf
OBJDUMP = riscv64-unknown-elf-objdump

AS_FLAGS = -march=rv32imf -Isrc -I../../sdk/c/inc
CC_FLAGS = -march=rv32imf -mabi=ilp32 -Isrc -I../../sdk/c/inc -ffunction-sections
LD_FLAGS = -T $(LD_SCRIPT) -nostdlib -b elf32-littleriscv
OBJCOPY_FLAGS = --output-target=binary

OBJ_DIR = obj
TARGET_DIR = bin
SRC_DIR = src

LD_SCRIPT = ../../sdk/link/link.ld

TARGET = cutout_blit.bin

SDK_PATH = ../../sdk
SDK_INC = $(SDK_PATH)/c/inc
SDK_LIB = $(SDK_PATH)/c/obj/sdk.o
SDK_MAKEFILE = $(SDK_PATH)/c/Makefile

default: build disassemble

$(OBJ_DIR):
	$(MKDIR) $(OBJ_DIR)

$(TARGET_DIR):
	$(MKDIR) $(TARGET_DIR)

$(OBJ_DIR)/startup.o: $(SRC_DIR) $(OBJ_DIR) $(SRC_DIR)/startup.o
	$(AS) $(AS_FLAGS) $(SRC_DIR)/startup.s -o $(OBJ_DIR)/startup.o

$(OBJ_DIR)/main.o: $(SRC_DIR) $(OBJ_DIR) $(SDK_INC)/dbg.h $(SRC_DIR)/main.c
	$(CC) $(CC_FLAGS) $(SRC_DIR)/main.c -c -o $(OBJ_DIR)/main.o

$(TARGET_DIR)/$(TARGET).elf: $(TARGET_DIR) $(OBJ_DIR)/main.o $(OBJ_DIR)/startup.o $(SDK_LIB)
	$(LD) $(LD_FLAGS) $(OBJ_DIR)/startup.o $(OBJ_DIR)/main.o $(SDK_LIB) -o $(TARGET_DIR)/$(TARGET).elf

$(TARGET_DIR)/$(TARGET): $(TARGET_DIR)/$(TARGET).elf
	$(OBJCOPY) $(OBJCOPY_FLAGS) $(TARGET_DIR)/$(TARGET).elf $(TARGET_DIR)/$(TARGET)

$(SDK_LIB):
	make -f $(SDK_MAKEFILE) sdk

disassembly.txt: $(TARGET_DIR)/$(TARGET).elf
	$(OBJDUMP) $(TARGET_DIR)/$(TARGET).elf --disassemble-all > disassembly.txt

disassemble: disassembly.txt

build: $(TARGET_DIR)/$(TARGET)

clean:
	make -f $(SDK_MAKEFILE) clean
	$(RMDIR) obj
	$(RMDIR) bin

rebuild: clean build

.phony: all clean default disassemble


