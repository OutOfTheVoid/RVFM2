SELF_DIR := $(dir $(lastword $(MAKEFILE_LIST)))

AS = riscv64-unknown-elf-as
LD = riscv64-unknown-elf-ld
CC = riscv64-unknown-elf-gcc
MKDIR = mkdir
RMDIR = rm -rf

LD_SCRIPT = $(SELF_DIR)../link/link.ld

AS_FLAGS = -march=rv32im_zicsr -Iinc
CC_FLAGS = -march=rv32im_zicsr -mabi=ilp32 -Isrc -I../../sdk/c/inc -ffunction-sections -fdata-sections
LD_FLAGS = -T $(LD_SCRIPT) -nostdlib -b elf32-littleriscv -A RISCV32

OBJ_DIR = $(SELF_DIR)obj
SRC_DIR = $(SELF_DIR)src
INC_DIR = $(SELF_DIR)inc

SDK_LIBNAME = sdk.o

$(OBJ_DIR):
	$(MKDIR) $(OBJ_DIR)

$(OBJ_DIR)/$(SDK_LIBNAME): $(OBJ_DIR)/string.o $(OBJ_DIR)
	$(LD) $(LD_FLAGS) $(OBJ_DIR)/string.o -r -o $(OBJ_DIR)/$(SDK_LIBNAME)

$(OBJ_DIR)/string.o: $(OBJ_DIR) $(SRC_DIR)/string.c $(INC_DIR)/string.h $(INC_DIR)/stdint.h $(INC_DIR)/types.h
	$(CC) $(CC_FLAGS) $(SRC_DIR)/string.c -c -o $(OBJ_DIR)/string.o

sdk: $(OBJ_DIR)/$(SDK_LIBNAME)

clean:
	$(RMDIR) $(OBJ_DIR)

.phony: sdk
