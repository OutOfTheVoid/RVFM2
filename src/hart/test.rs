
use crate::hart::decoder::FenceOps;

use super::decoder::Rv32Op;

const ZERO: u8 = 0;
const RA: u8 = 1;
const SP: u8 = 2;
const GP: u8 = 3;
const TP: u8 = 4;
const T0: u8 = 5;
const T1: u8 = 6;
const T2: u8 = 7;
const S0: u8 = 8;
const FP: u8 = 8;
const S1: u8 = 9;
const A0: u8 = 10;
const A1: u8 = 11;
const A2: u8 = 12;
const A3: u8 = 13;
const A4: u8 = 14;
const A5: u8 = 15;
const A6: u8 = 16;
const A7: u8 = 17;
const S2: u8 = 18;
const S3: u8 = 19;
const S4: u8 = 20;
const S5: u8 = 21;
const S6: u8 = 22;
const S7: u8 = 23;
const S8: u8 = 24;
const S9: u8 = 25;
const S10: u8 = 26;
const S11: u8 = 27;
const T3: u8 = 28;
const T4: u8 = 29;
const T5: u8 = 30;
const T6: u8 = 31;

#[test]
fn decoder_test() {
    let DECODE_TEST_LIST: &[(u32, Rv32Op)] = &[
        (0xFFFFF237, Rv32Op::Lui   { immediate: 0xFFFFF000,          rd: TP }),
        (0x00400297, Rv32Op::Auipc { immediate: 0x400000,            rd: T0 }),

        (0x000000EF, Rv32Op::Jal   { immediate: 0,                   rd: RA }),
        (0x020080E7, Rv32Op::Jalr  { immediate: 32,         rs1: RA, rd: RA }),

        (0x00209463, Rv32Op::Bne   { immediate: 8, rs2: SP, rs1: RA         }),
        (0x00538463, Rv32Op::Beq   { immediate: 8, rs2: T0, rs1: T2         }),
        (0x0053d463, Rv32Op::Bge   { immediate: 8, rs2: T0, rs1: T2         }),
        (0x0053c463, Rv32Op::Blt   { immediate: 8, rs2: T0, rs1: T2         }),
        (0x0053f463, Rv32Op::Bgeu  { immediate: 8, rs2: T0, rs1: T2         }),
        (0x0053e463, Rv32Op::Bltu  { immediate: 8, rs2: T0, rs1: T2         }),

        (0x00410083, Rv32Op::Lb    { immediate:     4,         rs1: SP, rd: RA }),
        (0x00811083, Rv32Op::Lh    { immediate:     8,         rs1: SP, rd: RA }),
        (0x00c12083, Rv32Op::Lw    { immediate:    12,         rs1: SP, rd: RA }),

        (0x00c14083, Rv32Op::Lbu   { immediate:    12, rd:  1, rs1:  2          }),
        (0x00c4d383, Rv32Op::Lhu   { immediate:    12, rd:  7, rs1:  9          }),

        (0x00478a23, Rv32Op::Sb    { immediate:    20,         rs1: 15, rs2:  4 }),
        (0x00581a23, Rv32Op::Sh    { immediate:    20,         rs1: 16, rs2:  5 }),
        (0x0068aa23, Rv32Op::Sw    { immediate:    20,         rs1: 17, rs2:  6 }),

        (0x00a18113, Rv32Op::Addi  { immediate:    10, rd:  2, rs1:  3          }),
        (0x00af2293, Rv32Op::Slti  { immediate:    10, rd:  5, rs1: 30          }),
        (0x1ff8b493, Rv32Op::Sltiu { immediate: 0x1FF, rd:  9, rs1: 17          }),
        (0x0011c113, Rv32Op::Xori  { immediate:     1, rd:  2, rs1:  3          }),
        (0x0021e113, Rv32Op::Ori   { immediate:     2, rd:  2, rs1:  3          }),
        (0x7ff1f113, Rv32Op::Andi  { immediate: 0x7FF, rd:  2, rs1:  3          }),
        
    ];

    let mut passed = 0;
    let mut failed = 0;
    for (codeword, correct_decode) in &DECODE_TEST_LIST[..] {
        let decode = Rv32Op::decode(*codeword);
        match decode == *correct_decode {
            true => {
                passed += 1;
            },
            false => {
                println!("decode failed: {:#010X} => {:?}, should be {:?}", codeword, decode, correct_decode);
                failed += 1;
            }
        }
    }
    println!("\ndecode test: {} passed, {} failed", passed, failed);
}

#[test]
fn test_disassembly() {
    let test_list = [ 
        (Rv32Op::Auipc { immediate: 0,      rd: T3                   }, 0x0000_0000, "auipc  t3,   00000000"      ),
        (Rv32Op::Lui   { immediate: 0x1000, rd: T0,                  }, 0x0000_0000, "lui    t0,   00001000"      ),
        (Rv32Op::Jal   { immediate: 0,      rd: T4,                  }, 0x0000_0000, "jal    t4,   00000000"      ),
        (Rv32Op::Jalr  { immediate: 12,     rd: T5, rs1: T6,         }, 0x0000_0000, "jalr   t5,   12(t6)"        ),
        
        (Rv32Op::Sw    { immediate: 48,             rs1: A0, rs2: A2 }, 0x0000_0000, "sw     a2,   48(a0)"        ),
        (Rv32Op::Sh    { immediate: 8,              rs1: A4, rs2: A5 }, 0x0000_0000, "sh     a5,   8(a4)"         ),
        (Rv32Op::Sb    { immediate: -8,             rs1: S8, rs2: S9 }, 0x0000_0000, "sb     s9,   -8(s8)"        ),
        
        (Rv32Op::Lw    { immediate: 4,      rd: S0, rs1: S1,         }, 0x0000_0000, "lw     s0,   4(s1)"         ),
        (Rv32Op::Lh    { immediate: 4,      rd: S2, rs1: S3,         }, 0x0000_0000, "lh     s2,   4(s3)"         ),
        (Rv32Op::Lb    { immediate: 4,      rd: S4, rs1: S5,         }, 0x0000_0000, "lb     s4,   4(s5)"         ),
        (Rv32Op::Lhu   { immediate: 4,      rd: S6, rs1: S7,         }, 0x0000_0000, "lhu    s6,   4(s7)"         ),
        (Rv32Op::Lbu   { immediate: 4,      rd: S8, rs1: S9,         }, 0x0000_0000, "lbu    s8,   4(s9)"         ),
        
        (Rv32Op::Addi  { immediate: 5,      rd: TP, rs1: RA,         }, 0x0000_0000, "addi   tp,   ra,   5"       ),
        (Rv32Op::Andi  { immediate: 0x104,  rd: GP, rs1: T1,         }, 0x0000_0000, "andi   gp,   t1,   00000104"),
        (Rv32Op::Xori  { immediate: 0x127,  rd: SP, rs1: S0,         }, 0x0000_0000, "xori   sp,   s0,   00000127"),
        (Rv32Op::Ori   { immediate: 0xFFF,  rd: T0, rs1: T1,         }, 0x0000_0000, "ori    t0,   t1,   00000FFF"),
        (Rv32Op::Slli  { shamt: 5,          rd: A0, rs1: A1,         }, 0x0000_0000, "slli   a0,   a1,   5"       ),
        (Rv32Op::Srli  { shamt: 7,          rd: A2, rs1: A3,         }, 0x0000_0000, "srli   a2,   a3,   7"       ),
        (Rv32Op::Srai  { shamt: 0,          rd: A4, rs1: A5,         }, 0x0000_0000, "srai   a4,   a5,   0"       ),

        (Rv32Op::Add   {                    rd: S0, rs1: S9, rs2: S4 }, 0x0000_0000, "add    s0,   s9,   s4"      ),
        (Rv32Op::Sub   {                    rd: S0, rs1: S9, rs2: S4 }, 0x0000_0000, "sub    s0,   s9,   s4"      ),
        (Rv32Op::And   {                    rd: S0, rs1: S9, rs2: S4 }, 0x0000_0000, "and    s0,   s9,   s4"      ),
        (Rv32Op::Xor   {                    rd: S0, rs1: S9, rs2: S4 }, 0x0000_0000, "xor    s0,   s9,   s4"      ),
        (Rv32Op::Or    {                    rd: S0, rs1: S9, rs2: S4 }, 0x0000_0000, "or     s0,   s9,   s4"      ),
        (Rv32Op::Sll   {                    rd: S0, rs1: S9, rs2: S4 }, 0x0000_0000, "sll    s0,   s9,   s4"      ),
        (Rv32Op::Srl   {                    rd: S0, rs1: S9, rs2: S4 }, 0x0000_0000, "srl    s0,   s9,   s4"      ),
        (Rv32Op::Sra   {                    rd: S0, rs1: S9, rs2: S4 }, 0x0000_0000, "sra    s0,   s9,   s4"      ),

        (Rv32Op::Beq   { immediate: 0x20,           rs1: T0, rs2: T1 }, 0x0000_0000, "beq    t0,   t1,   00000020"),
        (Rv32Op::Bne   { immediate: 0x20,           rs1: T0, rs2: T1 }, 0x0000_0000, "bne    t0,   t1,   00000020"),
        (Rv32Op::Bge   { immediate: 0x20,           rs1: T0, rs2: T1 }, 0x0000_0000, "bge    t0,   t1,   00000020"),
        (Rv32Op::Blt   { immediate: 0x20,           rs1: T0, rs2: T1 }, 0x0000_0000, "blt    t0,   t1,   00000020"),
        (Rv32Op::Bgeu  { immediate: 0x20,           rs1: T0, rs2: T1 }, 0x0000_0000, "bgeu   t0,   t1,   00000020"),
        (Rv32Op::Bltu  { immediate: 0x20,           rs1: T0, rs2: T1 }, 0x0000_0000, "bltu   t0,   t1,   00000020"),

        (Rv32Op::Wfi,                                                   0x0000_0000, "wfi"                        ),

        (Rv32Op::Fence { predecessor: FenceOps::RW,
                         successor:   FenceOps::IO                   }, 0x0000_0000, "fence  rw,   io"            ),
        (Rv32Op::Fencei,                                                0x0000_0000, "fence.i"                     ),

        (Rv32Op::Ecall,                                                 0x0000_0000, "ecall"                      ),
        (Rv32Op::EBreak,                                                0x0000_0000, "ebreak"                     ),

        (Rv32Op::Mul   {                    rd: S0, rs1: S9, rs2: S4 }, 0x0000_0000, "mul    s0,   s9,   s4"      ),
        (Rv32Op::Mulh  {                    rd: S0, rs1: S9, rs2: S4 }, 0x0000_0000, "mulh   s0,   s9,   s4"      ),
        (Rv32Op::Mulhsu{                    rd: S0, rs1: S9, rs2: S4 }, 0x0000_0000, "mulhsu s0,   s9,   s4"      ),
        (Rv32Op::Mulhu {                    rd: S0, rs1: S9, rs2: S4 }, 0x0000_0000, "mulhu  s0,   s9,   s4"      ),
        (Rv32Op::Div   {                    rd: S0, rs1: S9, rs2: S4 }, 0x0000_0000, "div    s0,   s9,   s4"      ),
        (Rv32Op::Divu  {                    rd: S0, rs1: S9, rs2: S4 }, 0x0000_0000, "divu   s0,   s9,   s4"      ),
        (Rv32Op::Rem   {                    rd: S0, rs1: S9, rs2: S4 }, 0x0000_0000, "rem    s0,   s9,   s4"      ),
        (Rv32Op::Remu  {                    rd: S0, rs1: S9, rs2: S4 }, 0x0000_0000, "remu   s0,   s9,   s4"      ),

        (Rv32Op::Csrrw { csr: 0x100,        rd: S0, rs1: S1,         }, 0x0000_0000, "csrrw  000,  s0,   s1"      ),
    ];
    let mut passed = 0;
    let mut failed = 0;
    for (op, addr, expected_disassembly) in &test_list[..] {
        let disassembly = op.assembly(*addr);
        if &disassembly == expected_disassembly {
            passed += 1;
        } else {
            println!("disassembly failed: {:?} disassembled to \"{}\", should be \"{}\"", op, disassembly, expected_disassembly);
            failed += 1;
        }
    }
    println!("\ndisassembly test: {} passed, {} failed", passed, failed);
}
