#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq)]
pub enum Rv32Op {
    Unknown,

    // RV32I Base instruction set
    Lui     {immediate: u32, rd: u8},
    Auipc  {immediate: u32, rd: u8},
  
    Jal     {immediate: u32, rd: u8},
    Jalr    {immediate: i32, rd: u8, rs1: u8},

    Beq     {immediate: i32, rs2: u8, rs1: u8},
    Bne     {immediate: i32, rs2: u8, rs1: u8},
    Blt     {immediate: i32, rs2: u8, rs1: u8},
    Bge     {immediate: i32, rs2: u8, rs1: u8},
    Bltu    {immediate: i32, rs2: u8, rs1: u8},
    Bgeu    {immediate: i32, rs2: u8, rs1: u8},

    Lb      {immediate: i32, rd: u8, rs1: u8},
    Lh      {immediate: i32, rd: u8, rs1: u8},
    Lw      {immediate: i32, rd: u8, rs1: u8},
    Lbu     {immediate: i32, rd: u8, rs1: u8},
    Lhu     {immediate: i32, rd: u8, rs1: u8},

    Sb      {immediate: i32, rs1: u8, rs2: u8},
    Sh      {immediate: i32, rs1: u8, rs2: u8},
    Sw      {immediate: i32, rs1: u8, rs2: u8},

    Addi    {immediate: i32, rd: u8, rs1: u8},
    Slti    {immediate: i32, rd: u8, rs1: u8},
    Sltiu   {immediate: u32, rd: u8, rs1: u8},
    Xori    {immediate: u32, rd: u8, rs1: u8},
    Ori     {immediate: u32, rd: u8, rs1: u8},
    Andi    {immediate: u32, rd: u8, rs1: u8},

    Slli    {shamt: u8, rd: u8, rs1: u8},
    Srli    {shamt: u8, rd: u8, rs1: u8},
    Srai    {shamt: u8, rd: u8, rs1: u8},

    Add     {rd: u8, rs1: u8, rs2: u8},
    Sub     {rd: u8, rs1: u8, rs2: u8},
    Sll     {rd: u8, rs1: u8, rs2: u8},
    Slt     {rd: u8, rs1: u8, rs2: u8},
    Sltu    {rd: u8, rs1: u8, rs2: u8},
    Xor     {rd: u8, rs1: u8, rs2: u8},
    Srl     {rd: u8, rs1: u8, rs2: u8},
    Sra     {rd: u8, rs1: u8, rs2: u8},
    Or      {rd: u8, rs1: u8, rs2: u8},
    And     {rd: u8, rs1: u8, rs2: u8},

    Fence   {predecessor: FenceOps, successor: FenceOps},
    
    Fencei,
    
    Ecall,
    EBreak,

    Csrrw   {csr: u16, rd: u8, rs1: u8},
    Csrrs   {csr: u16, rd: u8, rs1: u8},
    Csrrc   {csr: u16, rd: u8, rs1: u8},

    Csrrwi  {csr: u16, rd: u8, immediate: u8},
    Csrrsi  {csr: u16, rd: u8, immediate: u8},
    Csrrci  {csr: u16, rd: u8, immediate: u8},

    Wfi,
    Mret,

    // RV32M Multiply Divide extension
    Mul     {rd: u8, rs1: u8, rs2: u8},
    Mulh    {rd: u8, rs1: u8, rs2: u8},
    Mulhsu  {rd: u8, rs1: u8, rs2: u8},
    Mulhu   {rd: u8, rs1: u8, rs2: u8},
    Div     {rd: u8, rs1: u8, rs2: u8},
    Divu    {rd: u8, rs1: u8, rs2: u8},
    Rem     {rd: u8, rs1: u8, rs2: u8},
    Remu    {rd: u8, rs1: u8, rs2: u8},

    // RV32A Atomics
    Lr      {acquire: bool, release: bool,          rs1: u8, rd: u8},
    Sc      {acquire: bool, release: bool, rs2: u8, rs1: u8, rd: u8},
    AmoSwap {acquire: bool, release: bool, rs2: u8, rs1: u8, rd: u8},
    AmoAdd  {acquire: bool, release: bool, rs2: u8, rs1: u8, rd: u8},
    AmoXor  {acquire: bool, release: bool, rs2: u8, rs1: u8, rd: u8},
    AmoAnd  {acquire: bool, release: bool, rs2: u8, rs1: u8, rd: u8},
    AmoOr   {acquire: bool, release: bool, rs2: u8, rs1: u8, rd: u8},
    AmoMin  {acquire: bool, release: bool, rs2: u8, rs1: u8, rd: u8},
    AmoMax  {acquire: bool, release: bool, rs2: u8, rs1: u8, rd: u8},
    AmoMinU {acquire: bool, release: bool, rs2: u8, rs1: u8, rd: u8},
    AmoMaxU {acquire: bool, release: bool, rs2: u8, rs1: u8, rd: u8},
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq)]
pub struct FenceOps {
    pub input:  bool,
    pub output: bool,
    pub read:   bool,
    pub write:  bool,
}

impl FenceOps {
    pub const RW: Self = FenceOps {
        read:  true,  write:  true,
        input: false, output: false,
    };
    pub const IO: Self = FenceOps {
        read:  false, write:  false,
        input: true,  output: true,
    };
    pub const IORW: Self = FenceOps {
        read:  true, write:  true,
        input: true, output: true,
    };
}

fn field<const LOW: u32, const HIGH: u32>(x: u32) -> u32 {
    (x & (0xFFFF_FFFF >> (31 - HIGH))) >> LOW
}

fn jtype_immediate(x: u32) -> u32 {
    let bits = (field::<12, 19>(x) << 12) |
    (field::<20, 20>(x) << 11) |
    (field::<21, 30>(x) << 1) |
    (field::<31, 31>(x) << 20);
    if (bits & 0x80000) != 0 {
        bits | 0xFFF00000
    } else {
        bits
    }
}

fn btype_immediate(x: u32) -> i32 {
    let bits = 
        (field::<31, 31>(x) << 12) |
        (field::<25, 30>(x) <<  5) |
        (field::< 8, 11>(x) <<  1) |
        (field::< 7,  7>(x) << 11);
    if bits & (1 << 12) != 0 {
        (bits | 0xFFFFE000) as i32
    } else {
        bits as i32
    }
}

fn stype_immediate(x: u32) -> i32 {
    let bits = 
        (field::< 7, 11>(x) << 0) |
        (field::<25, 31>(x) << 5);
    (if (bits & 0x800) != 0 {
        bits | 0xFFFFF000
    } else {
        bits
    }) as i32
}

fn itype_immediate(x: u32) -> u32 {
    let bits = field::<20, 31>(x);
    if (bits & 0x800) != 0 {
        bits | 0xFFFF_F000
    } else {
        bits
    }
}

impl Rv32Op {
    pub fn decode(codeword: u32) -> Self {
        let opcode = field::<0, 6>(codeword);
        let funct3 = field::<12, 14>(codeword);
        match (opcode, funct3) {
            (0b0110111, _) => Self::Lui {
                immediate: codeword & 0xFFFFF000,
                rd:        field::<7, 11>(codeword) as u8
            },
            (0b0010111, _) => Self::Auipc {
                immediate: codeword & 0xFFFFF000,
                rd:        field::<7, 11>(codeword) as u8
            },
            (0b1101111, _) => Self::Jal {
                immediate: jtype_immediate(codeword),
                rd:        field::<7, 11>(codeword) as u8
            },
            (0b1100111, _) => {
                let funct3 = field::<12, 14>(codeword);
                let immediate = itype_immediate(codeword) as i32;
                match funct3 {
                    0b000 => Self::Jalr {
                        immediate,
                        rs1: field::<15, 19>(codeword) as u8,
                        rd:  field::< 7, 11>(codeword) as u8,
                    },
                    _    => Self::Unknown
                }
            },
            (0b1100011, funct3) => {
                let rs1 = field::<15, 19>(codeword) as u8;
                let rs2 = field::<20, 24>(codeword) as u8;
                let immediate = btype_immediate(codeword);
                match funct3 {
                    0b000 => Self::Beq  { immediate, rs1, rs2 },
                    0b001 => Self::Bne  { immediate, rs1, rs2 },
                    0b100 => Self::Blt  { immediate, rs1, rs2 },
                    0b101 => Self::Bge  { immediate, rs1, rs2 },
                    0b110 => Self::Bltu { immediate, rs1, rs2 },
                    0b111 => Self::Bgeu { immediate, rs1, rs2 },
                    _     => Self::Unknown
                }
            },
            (0b0000011, funct3) => {
                let rd = field::<7, 11>(codeword) as u8;
                let rs1 = field::<15, 19>(codeword) as u8;
                let immediate = itype_immediate(codeword) as i32;
                match funct3 {
                    0b000 => Self::Lb  { immediate, rd, rs1 },
                    0b001 => Self::Lh  { immediate, rd, rs1 },
                    0b010 => Self::Lw  { immediate, rd, rs1 },
                    0b100 => Self::Lbu { immediate, rd, rs1 },
                    0b101 => Self::Lhu { immediate, rd, rs1 },
                    _     => Self::Unknown,
                }
            },
            (0b0100011, funct3) => {
                let immediate = stype_immediate(codeword);
                let rs1 = field::<15, 19>(codeword) as u8;
                let rs2 = field::<20, 24>(codeword) as u8;
                match funct3 {
                    0b000 => Self::Sb { immediate, rs1, rs2 },
                    0b001 => Self::Sh { immediate, rs1, rs2 },
                    0b010 => Self::Sw { immediate, rs1, rs2 },
                    _     => Self::Unknown,
                }
            },
            (0b0010011, funct3) => {
                let funct7 = field::<25, 31>(codeword);

                let immediate = itype_immediate(codeword);
                let shamt = field::<20, 24>(codeword) as u8;

                let rd = field::<7, 11>(codeword) as u8;
                let rs1 = field::<15, 19>(codeword) as u8;

                match (funct7, funct3) {
                    (_,         0b000) => Self::Addi  { immediate: immediate as i32, rd, rs1 },
                    (_,         0b010) => Self::Slti  { immediate: immediate as i32, rd, rs1 },
                    (_,         0b011) => Self::Sltiu { immediate,                   rd, rs1 },
                    (_,         0b100) => Self::Xori  { immediate,                   rd, rs1 },
                    (_,         0b110) => Self::Ori   { immediate,                   rd, rs1 },
                    (_,         0b111) => Self::Andi  { immediate,                   rd, rs1 },
                    (0b0000000, 0b001) => Self::Slli  { shamt,                       rd, rs1 },
                    (0b0000000, 0b101) => Self::Srli  { shamt,                       rd, rs1 },
                    (0b0100000, 0b101) => Self::Srai  { shamt,                       rd, rs1 },
                    _                  => Self::Unknown,
                }
            },
            (0b0110011, funct3) => {
                let funct7 = field::<25, 31>(codeword);
                let rd = field::<7, 11>(codeword) as u8;
                let rs1 = field::<15, 19>(codeword) as u8;
                let rs2 = field::<20, 24>(codeword) as u8;
                match (funct7, funct3) {
                    (0b0000000, 0b000) => Self::Add  { rd, rs1, rs2 },
                    (0b0100000, 0b000) => Self::Sub  { rd, rs1, rs2 },
                    (0b0000000, 0b001) => Self::Sll  { rd, rs1, rs2 },
                    (0b0000000, 0b010) => Self::Slt  { rd, rs1, rs2 },
                    (0b0000000, 0b011) => Self::Sltu { rd, rs1, rs2 },
                    (0b0000000, 0b100) => Self::Xor  { rd, rs1, rs2 },
                    (0b0000000, 0b101) => Self::Srl  { rd, rs1, rs2 },
                    (0b0100000, 0b101) => Self::Sra  { rd, rs1, rs2 },
                    (0b0000000, 0b110) => Self::Or   { rd, rs1, rs2 },
                    (0b0000000, 0b111) => Self::And  { rd, rs1, rs2 },

                    (0b0000001, 0b000) => Self::Mul    { rd, rs1, rs2 },
                    (0b0000001, 0b001) => Self::Mulh   { rd, rs1, rs2 },
                    (0b0000001, 0b010) => Self::Mulhsu { rd, rs1, rs2 },
                    (0b0000001, 0b011) => Self::Mulhu  { rd, rs1, rs2 },
                    (0b0000001, 0b100) => Self::Div    { rd, rs1, rs2 },
                    (0b0000001, 0b101) => Self::Divu   { rd, rs1, rs2 },
                    (0b0000001, 0b110) => Self::Rem    { rd, rs1, rs2 },
                    (0b0000001, 0b111) => Self::Remu   { rd, rs1, rs2 },
                    _                  => Self::Unknown
                }
            },
            (0b0001111, funct3) => {
                let funct4 = field::<28, 31>(codeword);
                let pred = field::<24, 27>(codeword);
                let succ = field::<23, 20>(codeword);
                let rs1 = field::<15, 19>(codeword);
                let rd = field::<7, 11>(codeword);
                match (funct4, pred, succ, rs1, funct3, rd) {
                    (0b0000, pred, succ, 0b00000, 0b000, 0b00000) => {
                        let predecessor = FenceOps {
                            input:  (pred & 0b1000) != 0,
                            output: (pred & 0b0100) != 0,
                            read:   (pred & 0b0010) != 0,
                            write:  (pred & 0b0001) != 0,
                        };
                        let successor = FenceOps {
                            input:  (succ & 0b1000) != 0,
                            output: (succ & 0b0100) != 0,
                            read:   (succ & 0b0010) != 0,
                            write:  (succ & 0b0001) != 0,
                        };
                        Self::Fence { predecessor, successor }
                    },
                    (0b0000, 0b0000, 0b0000, 0b00000, 0b001, 0b00000) => Self::Fencei,
                    _                                                 => Self::Unknown,
                }
            },
            (0b1110011, 0b000) => {
                let bits_7_through_19 = field::<7, 19>(codeword);
                let funct12 = field::<20, 31>(codeword);
                match (funct12, bits_7_through_19) {
                    (0b000000000000, 0b0000000000000) => Self::Ecall,
                    (0b000000000001, 0b0000000000000) => Self::EBreak,
                    (0b000100000101, 0b0000000000000) => Self::Wfi,
                    (0b001100000010, 0b0000000000000) => Self::Mret,
                    _                                 => Self::Unknown,
                }
            },
            (0b1110011, funct3) => {
                let rs1 = field::<15, 19>(codeword) as u8;
                let immediate = rs1;
                let rd = field::<7, 11>(codeword) as u8;
                let csr = field::<20, 31>(codeword) as u16;
                match funct3 {
                    0b001 => Self::Csrrw  { csr, rd, rs1 },
                    0b010 => Self::Csrrs  { csr, rd, rs1 },
                    0b011 => Self::Csrrc  { csr, rd, rs1 },
                    0b101 => Self::Csrrwi { csr, rd, immediate },
                    0b110 => Self::Csrrsi { csr, rd, immediate },
                    0b111 => Self::Csrrci { csr, rd, immediate },
                    _     => Self::Unknown
                }
            },
            (0b0101111, 0b010) => {
                let funct5 = field::<27, 31>(codeword);
                let acquire = field::<26, 26>(codeword) != 0;
                let release = field::<25, 25>(codeword) != 0;
                let rs1 = field::<15, 19>(codeword) as u8;
                let rs2 = field::<20, 24>(codeword) as u8;
                let rd = field::<7, 11>(codeword) as u8;
                match (funct5, rs2) {
                    (0b00010, 0  ) => Self::Lr      { acquire, release, rs1,      rd },
                    (0b00011, rs2) => Self::Sc      { acquire, release, rs2, rs1, rd },
                    (0b00001, rs2) => Self::AmoSwap { acquire, release, rs2, rs1, rd },
                    (0b00000, rs2) => Self::AmoAdd  { acquire, release, rs2, rs1, rd },
                    (0b00100, rs2) => Self::AmoXor  { acquire, release, rs2, rs1, rd },
                    (0b01100, rs2) => Self::AmoAnd  { acquire, release, rs2, rs1, rd },
                    (0b01000, rs2) => Self::AmoOr   { acquire, release, rs2, rs1, rd },
                    (0b10000, rs2) => Self::AmoMin  { acquire, release, rs2, rs1, rd },
                    (0b10100, rs2) => Self::AmoMax  { acquire, release, rs2, rs1, rd },
                    (0b11000, rs2) => Self::AmoMinU { acquire, release, rs2, rs1, rd },
                    (0b11100, rs2) => Self::AmoMaxU { acquire, release, rs2, rs1, rd },
                    _ => Self::Unknown,
                }
            },
            _ => Self::Unknown,
        }
    }

    pub fn register_name(r: u8) -> &'static str {
        match r {
            0 =>  "zero",
            1 =>  "ra",
            2 =>  "sp",
            3 =>  "gp",
            4 =>  "tp",
            5 =>  "t0",
            6 =>  "t1",
            7 =>  "t2",
            8 =>  "s0",
            9 =>  "s1",
            10 => "a0",
            11 => "a1",
            12 => "a2",
            13 => "a3",
            14 => "a4",
            15 => "a5",
            16 => "a6",
            17 => "a7",
            18 => "s2",
            19 => "s3",
            20 => "s4",
            21 => "s5",
            22 => "s6",
            23 => "s7",
            24 => "s8",
            25 => "s9",
            26 => "s10",
            27 => "s11",
            28 => "t3",
            29 => "t4",
            30 => "t5",
            31 => "t6",
            _ =>  "?"
        }
    }

    fn assembly_r_r_offset(instruction_name: &str, r: u8, r_off: u8, offset: i32) -> String {
        let r_string = format!("{},", Self::register_name(r));
        let r_offset_string = format!("({}){}", offset, Self::register_name(r_off));
        format!("{:<5} {:<5} {}", instruction_name, r_string, r_offset_string)
    }

    fn csr_name(csr: u16) -> &'static str {
        match csr {

            _ => "?"
        }
    }

    pub fn assembly(&self, address: u32) -> String {
        match self {
            &Rv32Op::Auipc { immediate, rd            } => format!("auipc  {:<5} {:08X}",  format!("{},", Self::register_name(rd)), immediate),
            &Rv32Op::Lui   { immediate, rd,           } => format!("lui    {:<5} {:08X}", format!("{},", Self::register_name(rd)), immediate),

            &Rv32Op::Jal   { immediate, rd,           } => format!("jal    {:<5} {:08X}", format!("{},", Self::register_name(rd)), address.wrapping_add(immediate)),
            &Rv32Op::Jalr  { immediate, rd, rs1       } => format!("jalr   {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{}({})",  immediate, Self::register_name(rs1))),

            &Rv32Op::Sw    { immediate,     rs1, rs2, } => format!("sw     {:<5} {}",  format!("{},", Self::register_name(rs2)), format!("{}({})",  immediate, Self::register_name(rs1))),
            &Rv32Op::Sh    { immediate,     rs1, rs2, } => format!("sh     {:<5} {}",  format!("{},", Self::register_name(rs2)), format!("{}({})",  immediate, Self::register_name(rs1))),
            &Rv32Op::Sb    { immediate,     rs1, rs2, } => format!("sb     {:<5} {}",  format!("{},", Self::register_name(rs2)), format!("{}({})",  immediate, Self::register_name(rs1))),

            &Rv32Op::Lw    { immediate, rd, rs1,      } => format!("lw     {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{}({})",  immediate, Self::register_name(rs1))),
            &Rv32Op::Lh    { immediate, rd, rs1,      } => format!("lh     {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{}({})",  immediate, Self::register_name(rs1))),
            &Rv32Op::Lb    { immediate, rd, rs1,      } => format!("lb     {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{}({})",  immediate, Self::register_name(rs1))),
            &Rv32Op::Lhu   { immediate, rd, rs1,      } => format!("lhu    {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{}({})",  immediate, Self::register_name(rs1))),
            &Rv32Op::Lbu   { immediate, rd, rs1,      } => format!("lbu    {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{}({})",  immediate, Self::register_name(rs1))),
            
            &Rv32Op::Addi  { immediate, rd, rs1,      } => format!("addi   {:<5} {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), immediate),
            &Rv32Op::Andi  { immediate, rd, rs1,      } => format!("andi   {:<5} {:<5} {:08X}",  format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), immediate),
            &Rv32Op::Xori  { immediate, rd, rs1,      } => format!("xori   {:<5} {:<5} {:08X}",  format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), immediate),
            &Rv32Op::Ori   { immediate, rd, rs1,      } => format!("ori    {:<5} {:<5} {:08X}",  format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), immediate),
            &Rv32Op::Slli  { shamt,     rd, rs1,      } => format!("slli   {:<5} {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), shamt),
            &Rv32Op::Srli  { shamt,     rd, rs1,      } => format!("srli   {:<5} {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), shamt),
            &Rv32Op::Srai  { shamt,     rd, rs1,      } => format!("srai   {:<5} {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), shamt),

            &Rv32Op::Add   {            rd, rs1, rs2, } => format!("add    {:<5} {:<5} {}",     format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), Self::register_name(rs2)),
            &Rv32Op::Sub   {            rd, rs1, rs2, } => format!("sub    {:<5} {:<5} {}",     format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), Self::register_name(rs2)),
            &Rv32Op::And   {            rd, rs1, rs2, } => format!("and    {:<5} {:<5} {}",     format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), Self::register_name(rs2)),
            &Rv32Op::Xor   {            rd, rs1, rs2, } => format!("xor    {:<5} {:<5} {}",     format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), Self::register_name(rs2)),
            &Rv32Op::Or    {            rd, rs1, rs2, } => format!("or     {:<5} {:<5} {}",     format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), Self::register_name(rs2)),
            &Rv32Op::Sll   {            rd, rs1, rs2, } => format!("sll    {:<5} {:<5} {}",     format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), Self::register_name(rs2)),
            &Rv32Op::Srl   {            rd, rs1, rs2, } => format!("srl    {:<5} {:<5} {}",     format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), Self::register_name(rs2)),
            &Rv32Op::Sra   {            rd, rs1, rs2, } => format!("sra    {:<5} {:<5} {}",     format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), Self::register_name(rs2)),

            &Rv32Op::Beq   { immediate,     rs1, rs2, } => format!("beq    {:<5} {:<5} {:08X}", format!("{},", Self::register_name(rs1)), format!("{},", Self::register_name(rs2)), address.wrapping_add(immediate as u32)),
            &Rv32Op::Bne   { immediate,     rs1, rs2, } => format!("bne    {:<5} {:<5} {:08X}", format!("{},", Self::register_name(rs1)), format!("{},", Self::register_name(rs2)), address.wrapping_add(immediate as u32)),
            &Rv32Op::Bge   { immediate,     rs1, rs2, } => format!("bge    {:<5} {:<5} {:08X}", format!("{},", Self::register_name(rs1)), format!("{},", Self::register_name(rs2)), address.wrapping_add(immediate as u32)),
            &Rv32Op::Blt   { immediate,     rs1, rs2, } => format!("blt    {:<5} {:<5} {:08X}", format!("{},", Self::register_name(rs1)), format!("{},", Self::register_name(rs2)), address.wrapping_add(immediate as u32)),
            &Rv32Op::Bgeu  { immediate,     rs1, rs2, } => format!("bgeu   {:<5} {:<5} {:08X}", format!("{},", Self::register_name(rs1)), format!("{},", Self::register_name(rs2)), address.wrapping_add(immediate as u32)),
            &Rv32Op::Bltu  { immediate,     rs1, rs2, } => format!("bltu   {:<5} {:<5} {:08X}", format!("{},", Self::register_name(rs1)), format!("{},", Self::register_name(rs2)), address.wrapping_add(immediate as u32)),

            &Rv32Op::Wfi                                => format!("wfi"),

            &Rv32Op::Ecall                              => format!("ecall"),
            &Rv32Op::EBreak                             => format!("ebreak"),
            
            &Rv32Op::Mul   {            rd, rs1, rs2, } => format!("mul    {:<5} {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), Self::register_name(rs2)),
            &Rv32Op::Mulh  {            rd, rs1, rs2, } => format!("mulh   {:<5} {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), Self::register_name(rs2)),
            &Rv32Op::Mulhsu{            rd, rs1, rs2, } => format!("mulhsu {:<5} {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), Self::register_name(rs2)),
            &Rv32Op::Mulhu {            rd, rs1, rs2, } => format!("mulhu  {:<5} {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), Self::register_name(rs2)),
            &Rv32Op::Div   {            rd, rs1, rs2, } => format!("div    {:<5} {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), Self::register_name(rs2)),
            &Rv32Op::Divu  {            rd, rs1, rs2, } => format!("divu   {:<5} {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), Self::register_name(rs2)),
            &Rv32Op::Rem   {            rd, rs1, rs2, } => format!("rem    {:<5} {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), Self::register_name(rs2)),
            &Rv32Op::Remu  {            rd, rs1, rs2, } => format!("remu   {:<5} {:<5} {}",  format!("{},", Self::register_name(rd)), format!("{},", Self::register_name(rs1)), Self::register_name(rs2)),

            &Rv32Op::Fencei                                         => format!("fence.i"),
            &Rv32Op::Fence { predecessor, successor               } => {
                let pred_string = format!("{}{}{}{},",
                    if predecessor.input {"i"} else {""},
                    if predecessor.output{"o"} else {""},
                    if predecessor.read  {"r"} else {""},
                    if predecessor.write {"w"} else {""},
                );
                let succ_string = format!("{}{}{}{}",
                    if successor.input {"i"} else {""},
                    if successor.output{"o"} else {""},
                    if successor.read  {"r"} else {""},
                    if successor.write {"w"} else {""},
                );
                                                                       format!("fence  {:<5} {}", pred_string, succ_string)
            },

            &Rv32Op::Csrrwi { csr, rd, immediate } =>                  format!("csrrwi {:<5}, {:<6}, {:2X}", Self::register_name(rd), Self::csr_name(csr), immediate),
            &Rv32Op::Csrrsi { csr, rd, immediate } =>                  format!("csrrsi {:<5}, {:<6}, {:2X}", Self::register_name(rd), Self::csr_name(csr), immediate),
            &Rv32Op::Csrrci { csr, rd, immediate } =>                  format!("csrrci {:<5}, {:<6}, {:2X}", Self::register_name(rd), Self::csr_name(csr), immediate),

            _                                   => format!("unimplemented disassembly: {:?}", self),
        }
    }
}

