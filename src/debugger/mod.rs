use std::{sync::{Arc, mpsc::{Receiver, self, TryRecvError}}, io::{Write, Stdout, Stdin, Read}, fs::File, collections::{HashMap, BTreeMap}, path::Path, ops::{Bound, Sub}};

use elf_utilities::{*, file::{ELF, ELF32}, symbol::Symbol32, section::Contents32};
use gdbstub_arch::riscv::Riscv32;
use noline::{sync::Editor, builder::EditorBuilder};
use std::io;
use termion::raw::IntoRawMode;


use crate::{machine::{Machine, ReadResult}, hart::{Hart, StepState, self, decoder::Rv32Op, csrs::SharedCSRs}, debugger::command::DataType};

use self::command::{Command, BreakpointName, ListType};

mod command;

#[derive(Copy, Clone, Debug, PartialEq)]
enum HartExecutionMode {
    Running,
    SingleStepping,
    WaitingForInterrupt,
    HitBreakpoint,
    Stopped,
    Faulted,
}

struct Breakpoint {
    pub address: u32,
    pub symbol: Option<String>,
    pub id: usize,
}

pub struct Debugger {
    harts                 : [Hart; 4],
    exec_modes            : [HartExecutionMode; 4],
    alive                 : bool,
    machine               : Arc<Machine>,
    breakpoints           : Vec<Breakpoint>,
    breakpoint_id_counter : usize,
}

struct StdinLineStream {
    line_rx: Receiver<String>,
}

impl StdinLineStream {
    pub fn new() -> Self {
        let (line_tx, line_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut io = noline::sync::std::IO::new(
                std::io::stdin(),
                std::io::stdout().into_raw_mode().unwrap()
            );
            let mut editor = noline::builder::EditorBuilder::new_unbounded()
                .with_unbounded_history()
                .build_sync(&mut io)
                .unwrap();
            let mut prompt = "> ";
            loop {
                match editor.readline(prompt, &mut io) {
                    Ok(line) => line_tx.send(line.to_owned()).unwrap(),
                    Err(_) => {},
                }
                prompt = "";
            }
        });
        Self {
            line_rx
        }
    }

    pub fn wait(&mut self) -> String {
        self.line_rx.recv().unwrap()
    }

    pub fn check(&mut self) -> Option<String> {
        match self.line_rx.try_recv() {
            Ok(line) => Some(line),
            Err(TryRecvError::Empty) => None,
            Err(_) => panic!("channel closed!")
        }
    }
}

impl Debugger {
    pub fn new(machine: Arc<Machine>) -> Self {
        let shared_csrs = SharedCSRs::new();
        Self {
            harts: [
                Hart::new(0xF800_0000, 0, &shared_csrs, &machine),
                Hart::new(0xF800_0000, 1, &shared_csrs, &machine),
                Hart::new(0xF800_0000, 2, &shared_csrs, &machine),
                Hart::new(0xF800_0000, 3, &shared_csrs, &machine),
            ],
            machine,
            alive: true,
            exec_modes: [
                HartExecutionMode::Stopped,
                HartExecutionMode::Stopped,
                HartExecutionMode::Stopped,
                HartExecutionMode::Stopped,
            ],
            breakpoints: Vec::new(),
            breakpoint_id_counter: 0,
        }
    }

    fn prompt(stdout: &mut Stdout) {
        print!("\r\n> ");
        stdout.flush().unwrap();
    }

    fn print_instruction(machine: &Arc<Machine>, address: u32, current: bool) {
        let opcode_value = match machine.read_u32(address) {
            ReadResult::Ok(value) => value,
            _ => 0
        };
        let opcode = hart::decoder::Rv32Op::decode(opcode_value);
        if current {
            print!("> {:08X}: {:08X} {}\r\n", address, opcode_value, opcode.assembly(address));
        } else {
            print!("  {:08X}: {:08X} {}\r\n", address, opcode_value, opcode.assembly(address));
        }
    }

    fn print_code_region(machine: &Arc<Machine>, address: u32, symbol_map: &BTreeMap<u32, (String, symbol::Type)>) {
        let code_start_addr = address.wrapping_sub(8);
        let (symbol_string, start_addr, end_addr) = 
            if let Some((&symbol_address, symbol)) = symbol_map.range(..=address).last() {
                if code_start_addr > symbol_address {
                    let offset = code_start_addr.wrapping_sub(symbol_address);
                    (
                        Some(format!("{} + {}:", symbol.0, offset)),
                        code_start_addr,
                        code_start_addr.wrapping_add(20)
                    )
                } else {
                    (
                        Some(format!("{}:", symbol.0)),
                        symbol_address,
                        symbol_address.wrapping_add(20)
                    )
                }
            } else {
                (
                    None,
                    code_start_addr,
                    code_start_addr.wrapping_add(20)
                )
            };
        let mut addr = start_addr;
        if let Some(symbol_string) = symbol_string {
            print!("  {symbol_string}\r\n\r\n");
        }
        while addr < end_addr {
            Self::print_instruction(machine, addr, addr == address);
            addr += 4;
        }
    }

    fn symbol_type_value(t: &symbol::Type) -> u32 {
        match t {
            symbol::Type::Func => 3,
            symbol::Type::Object => 2,
            symbol::Type::NoType => 1,
            _ => 0
        }
    }

    pub fn run(mut self, file_path: Option<&str>) {
        print!("rvfm debugger\r\n");
        let mut symbol_table: HashMap<String, u32> = HashMap::new();
        let mut symbol_tree: BTreeMap<u32, (String, symbol::Type)> = BTreeMap::new();
        let mut stdout = std::io::stdout();
        let mut line_stream = StdinLineStream::new();
        let mut last_execution_modes = [HartExecutionMode::Stopped; 4];
        let mut last_breakpoint_addrs = [3; 4];
        let mut hart_breakpoints = [0usize; 4];
        let mut current_hart = 0;
        let mut print_prompt = true;
        if let Some(file_path) = file_path {
            match elf_utilities::parser::parse_elf32(file_path) {
                Ok(elf) => {
                    for section in elf.sections.iter() {
                        match &section.contents {
                            Contents32::Symbols(symbols) => {
                                for symbol in &symbols[..] {
                                    let addr = symbol.st_value;
                                    let name = symbol.symbol_name.clone();
                                    let stype = symbol.get_type();
                                    let insert = if let Some(existing) = symbol_tree.get(&addr) {
                                        Self::symbol_type_value(&stype) > Self::symbol_type_value(&existing.1)
                                    } else {
                                        true
                                    };
                                    if insert {
                                        symbol_tree.insert(addr, (name.clone(), stype));
                                        symbol_table.insert(name, addr);
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                },
                Err(error) => {
                    print!("failed to read elf file: {}\r\n", error.to_string());
                }
            }
        }

        let mut last_command = None;
        loop {
            for hart in 0..4 {
                if self.exec_modes[hart] == HartExecutionMode::Running || self.exec_modes[hart] == HartExecutionMode::SingleStepping {
                    let pc = self.harts[hart].pc;
                    let mut hit_breakpoint = false;
                    for (i, breakpoint) in self.breakpoints.iter().enumerate() {
                        if pc == breakpoint.address && !(last_execution_modes[hart] == HartExecutionMode::HitBreakpoint && last_breakpoint_addrs[hart] == pc) {
                            self.exec_modes[hart] = HartExecutionMode::HitBreakpoint;
                            last_breakpoint_addrs[hart] = pc;
                            hart_breakpoints[hart] = i;
                            hit_breakpoint = true;
                            print!("hart {hart} hit breakpoint: {:08X}\r\n", pc);
                            break;
                        }
                    }
                    if !hit_breakpoint {
                        match self.harts[hart].single_step::<false>() {
                            StepState::Run => {},
                            StepState::WaitForInterrupt => self.exec_modes[hart] = HartExecutionMode::Stopped,
                            StepState::InstructionError |
                            StepState::BusError => self.exec_modes[hart] = HartExecutionMode::Faulted,
                        }
                    }
                    if self.exec_modes[hart] == HartExecutionMode::SingleStepping {
                        self.exec_modes[hart] = HartExecutionMode::Stopped;
                        last_execution_modes[hart] = HartExecutionMode::SingleStepping;
                    }
                }
            }

            let mut stop = false;
            for hart in 0..4 {
                if self.exec_modes[hart] != last_execution_modes[hart] {
                    stop |= match self.exec_modes[hart] {
                        HartExecutionMode::Stopped => {
                            print!("\nhart {} stopped! pc: {:08X}\r\n\r\n", hart, self.harts[hart].pc);
                            Self::print_code_region(&self.machine, self.harts[hart].pc, &symbol_tree);
                            print_prompt = true;
                            true
                        },
                        HartExecutionMode::Faulted => {
                            print!("\nhart {} hit a fault!\r\n\r\n", hart);
                            Self::print_code_region(&self.machine, self.harts[hart].pc, &symbol_tree);
                            print_prompt = true;
                            true
                        },
                        HartExecutionMode::HitBreakpoint => {
                            print!("\nhart {} hit breakpoint!\r\n\r\n", hart);
                            Self::print_code_region(&self.machine, self.harts[hart].pc, &symbol_tree);
                            print_prompt = true;
                            true
                        },
                        HartExecutionMode::WaitingForInterrupt => {
                            print!("\nhart {} waiting for interrupt...\r\n\r\n", hart);
                            Self::print_code_region(&self.machine, self.harts[hart].pc, &symbol_tree);
                            print_prompt = true;
                            false
                        },
                        _ => false
                    };
                }
            }
            if stop {
                for hart in 0..4 {
                    match self.exec_modes[hart] {
                        HartExecutionMode::Running | HartExecutionMode::SingleStepping => {
                            self.exec_modes[hart] = HartExecutionMode::Stopped;
                            last_execution_modes[hart] = HartExecutionMode::Stopped;
                        },
                        x @ _ => {
                            last_execution_modes[hart] = x;
                        }
                    }
                }
            } else {
                last_execution_modes = self.exec_modes;
            }
            

            let mut harts_running = false;
            if self.alive {
                for exec_mode in self.exec_modes.iter() {
                    if exec_mode == &HartExecutionMode::Running || exec_mode == &HartExecutionMode::SingleStepping {
                        harts_running = true;
                    }
                }
            }

            if print_prompt {
                Self::prompt(&mut stdout);
                print_prompt = false;
            }

            if let Some(line) = if harts_running { line_stream.check() } else { Some(line_stream.wait()) } {
                let line = line.trim().to_string();
                let command = if let Some(command) = Command::parse(&line) {
                    Some(command)
                } else {
                    if line.len() == 0 { last_command.clone() } else { None }
                };

                last_command = command.clone();
                match command {
                    Some(Command::Help(topic)) => {
                        self.help(topic);
                    },
                    Some(Command::Kill) => {
                        self.alive = false;
                        // EVENTUALLY: stop execution of peripherals, clean up runtime state
                    },
                    Some(Command::Quit) => {
                        return;
                    }
                    Some(Command::SetBreakpoint(BreakpointName::Address(addr))) => {
                        let id = self.breakpoint_id_counter;
                        self.breakpoint_id_counter += 1;
                        self.breakpoints.push(Breakpoint {
                            address: addr,
                            symbol: None,
                            id
                        });
                        print!("added breakpoint {} at address {:08X}\r\n", id, addr);
                    },
                    Some(Command::SetBreakpoint(BreakpointName::Symbol(symbol_name))) => {
                        if let Some(symbol_addr) = symbol_table.get(&symbol_name) {
                            let id = self.breakpoint_id_counter;
                            self.breakpoint_id_counter += 1;
                            self.breakpoints.push(Breakpoint {
                                address: *symbol_addr,
                                symbol: Some(symbol_name.clone()),
                                id
                            });
                            print!("added breakpoint {} at {} ({:08X})\r\n", id, symbol_name, symbol_addr);
                        }
                    }
                    Some(Command::ClearBreakpoint(BreakpointName::Id(id))) => {
                        if let Some(index) = self.breakpoints.iter().position(|b| b.id == id) {
                            print!("removed breakpoint {}\r\n", id);
                            self.breakpoints.remove(index);
                        }
                    },
                    Some(Command::ClearBreakpoint(BreakpointName::Address(addr))) => {
                        if let Some(index) = self.breakpoints.iter().position(|b| b.address == addr) {
                            print!("removed breakpoint {}\r\n", self.breakpoints[index].id);
                            self.breakpoints.remove(index);
                        }
                    },
                    Some(Command::ClearBreakpoint(BreakpointName::Symbol(symbol_name))) => {
                        if let Some(symbol_addr) = symbol_table.get(&symbol_name) {
                            if let Some(index) = self.breakpoints.iter().position(|b| b.address == *symbol_addr) {
                                print!("removed breakpoint {}\r\n", self.breakpoints[index].id);
                                self.breakpoints.remove(index);
                            }
                        }
                    },
                    Some(Command::List(types)) => {
                        for list_type in types {
                            match list_type {
                                ListType::Breakpoints => {
                                    for breakpoint in self.breakpoints.iter() {
                                        match breakpoint.symbol.as_ref() {
                                            Some(symbol_name) => print!("{}: {:08X} {}\r\n", breakpoint.id, breakpoint.address, symbol_name),
                                            None                       => print!("{}: {:08X}\r\n",    breakpoint.id, breakpoint.address),
                                        }
                                        
                                    }
                                },
                                ListType::Symbols => {
                                    for (&addr, symbol) in symbol_tree.iter() {
                                        print!("{:08x} {}", addr, symbol.0);
                                    }
                                }
                            }
                        }
                    },
                    Some(Command::Stop(harts_to_stop)) => {
                        for hart in 0..4 {
                            match (harts_to_stop[hart], self.exec_modes[hart]) {
                                (true, HartExecutionMode::Running | HartExecutionMode::WaitingForInterrupt | HartExecutionMode::HitBreakpoint) => {
                                    self.exec_modes[hart] = HartExecutionMode::Stopped;
                                    last_execution_modes[hart] = HartExecutionMode::Stopped;
                                    print!("hart {} stopped\r\n", hart);
                                },
                                (true, HartExecutionMode::Stopped) => {
                                    print!("hart {} already stopped\r\n", hart);
                                }
                                _ => {}
                            }
                        }
                    },
                    Some(Command::Continue(harts_to_continue)) => {
                        for hart in 0..4 {
                            match (harts_to_continue[hart], self.exec_modes[hart]) {
                                (true, HartExecutionMode::Stopped | HartExecutionMode::WaitingForInterrupt) => {
                                    self.exec_modes[hart] = HartExecutionMode::Running;
                                    last_execution_modes[hart] = HartExecutionMode::Running;
                                },
                                (true, HartExecutionMode::HitBreakpoint) => {
                                    self.exec_modes[hart] = HartExecutionMode::Running;
                                    last_execution_modes[hart] = HartExecutionMode::HitBreakpoint;
                                }
                                _ => {}
                            }
                        }
                    },
                    Some(Command::Hart(hart)) => {
                        if hart != current_hart {
                            print!("switched to hart {}\r\n", hart);
                            current_hart = hart;
                        } else {
                            print!("already on hart {}\r\n", hart);
                        }
                    },
                    Some(Command::Regs) => {
                        for r in 0..32 {
                            if r & 3 == 3 {
                                print!("{:<5} {:08X}\r\n", format!("{}:", Rv32Op::register_name(r)), self.harts[current_hart].gprs[r as usize]);
                            } else {
                                print!("{:<5} {:08X}  ", format!("{}:", Rv32Op::register_name(r)), self.harts[current_hart].gprs[r as usize]);
                            }
                        }
                        print!("pc:   {:08X}\r\n", self.harts[current_hart].pc);
                    },
                    Some(Command::SingleStep) => {
                        match self.exec_modes[current_hart] {
                            HartExecutionMode::Stopped | HartExecutionMode::WaitingForInterrupt => {
                                last_execution_modes[current_hart] = HartExecutionMode::Stopped;
                                self.exec_modes[current_hart] = HartExecutionMode::SingleStepping;
                            },
                            HartExecutionMode::HitBreakpoint => {
                                last_execution_modes[current_hart] = HartExecutionMode::HitBreakpoint;
                                self.exec_modes[current_hart] = HartExecutionMode::SingleStepping;
                            },
                            _ => {}
                        }
                    },
                    Some(Command::Read(DataType::U8, addr)) => {
                        match self.machine.read_u8(addr) {
                            ReadResult::Ok(value) => print!("{:02X}", value),
                            _ => print!("failed to read from {:08X}\r\n", addr),
                        }
                    },
                    Some(Command::Read(DataType::U16, addr)) => {
                        match self.machine.read_u16(addr) {
                            ReadResult::Ok(value) => print!("{:04X}", value),
                            _ => print!("failed to read from {:08X}\r\n", addr),
                        }
                    },
                    Some(Command::Read(DataType::U32, addr)) => {
                        match self.machine.read_u32(addr) {
                            ReadResult::Ok(value) => print!("{:08X}", value),
                            _ => print!("failed to read from {:08X}\r\n", addr),
                        }
                    },
                    Some(other) => {
                        print!("unimplemented command: {:?}\r\n", other);
                    },
                    None => {
                        if line.len() > 0 {
                            print!("failed to parse command: {:?}\r\n", line);
                        }
                    }
                }
                print_prompt = true;
            }
        }
    }

    pub fn help(&self, topic: Option<String>) {
        if let Some(topic) = topic {
            match topic.as_str() {
                "b" | "breakpoints" => {
                    print!("\r\n");
                },
                "r" | "running" => {
                    print!("\r\n")
                },
                topic => {
                    print!("unknown help topic: {}\r\n", topic);
                }
            }
        } else {
            print!("commands:\r\n");
            print!("    - help (topic)                 : print help information (optionally for a topic)\r\n");
            print!("          aliases                  : help h\r\n");
            print!("          help topics              :\r\n");
            print!("              * b | breakpoints\r\n");
            print!("              * r | running\r\n");
            print!("\r\n");
            print!("    - quit                         : quits rvfm\r\n");
            print!("          aliases                  : quit q\r\n");
            print!("\r\n");
            print!("    - kill                         : kill the machine, stopping execution\r\n");
            print!("          aliases                  : kill k\r\n");
            print!("\r\n");
            print!("    - list [<list name>]           : enumerate members of a list\r\n");
            print!("          aliases                  : list l\r\n");
            print!("          lists                    :\r\n");
            print!("               * breakpoints\r\n");
            print!("               * symbol\r\n");
            print!("\r\n");
            print!("    - continue <hart> ([, <hart>]) : continues the targeted hart(s)\r\n");
            print!("          aliases                  : continue c\r\n");
            print!("          harts                    :\r\n");
            print!("              * 0\r\n");
            print!("              * 1\r\n");
            print!("              * 2\r\n");
            print!("              * 3\r\n");
            print!("              * all\r\n");
            print!("\r\n");
            print!("    - stop <hart> [(, <hart>)]     : stops the targeted hart(s)\r\n");
            print!("          aliases                  : stop S\r\n");
            print!("          harts                    :\r\n");
            print!("              * 0\r\n");
            print!("              * 1\r\n");
            print!("              * 2\r\n");
            print!("              * 3\r\n");
            print!("              * all\r\n");
            print!("\r\n");
            print!("    - hart <hart>                  : stops the targeted hart(s)\r\n");
            print!("          aliases                  : hart H\r\n");
            print!("          harts                    :\r\n");
            print!("              * 0\r\n");
            print!("              * 1\r\n");
            print!("              * 2\r\n");
            print!("              * 3\r\n");
            print!("\r\n");
            print!("    - set_bp <breakpoint>          : creates a breakpoint at the address/symbol specified\r\n");
            print!("          aliases                  : set_bp b\r\n");
            print!("          breakpoints              :\r\n");
            print!("              * <symbol name>\r\n");
            print!("              * @ <hex address>\r\n");
            print!("\r\n");
            print!("    - clear_bp <breakpoint>        : creates a breakpoint at the address/symbol specified\r\n");
            print!("          aliases                  : clear_bp !b\r\n");
            print!("          breakpoints              :\r\n");
            print!("              * <symbol name>\r\n");
            print!("              * @ <hex address>\r\n");
            print!("              * <breakpoint id>\r\n");
            print!("\r\n");
            print!("    - step                         : single-steps the current hart\r\n");
            print!("          aliases                  : step s\r\n");
            print!("\r\n");
            print!("    - regs                         : prints registers for the current hart\r\n");
            print!("          aliases                  : regs r\r\n");
        }
    }
}
