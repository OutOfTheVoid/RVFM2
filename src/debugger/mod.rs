use std::{sync::{Arc, mpsc::{Receiver, self, TryRecvError}}, io::{Stdout, Stdin, Read}, fs::File, collections::{HashMap, BTreeMap}, path::Path, ops::{Bound, Sub}};

use elf_utilities::{*, file::{ELF, ELF32}, symbol::Symbol32, section::Contents32};
use gdbstub_arch::riscv::Riscv32;
use noline::{builder::EditorBuilder, sync::{Editor, Write as _}};
use parking_lot::{Condvar, Mutex};
use std::io;
use termion::raw::IntoRawMode;
use std::fmt::Write;


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

enum StdinLineStreamMessage {
    Prompt,
    Write(String),
    Shutdown,
}

struct StdinLineStream {
    line_rx: Receiver<String>,
    message_tx: mpsc::Sender<StdinLineStreamMessage>
}

impl StdinLineStream {
    pub fn new() -> Self {
        let (line_tx, line_rx) = mpsc::channel();
        let prompt_mutex = Arc::new(Mutex::new(true));
        let prompt_cv = Arc::new(Condvar::new());
        let thread_prompt_mutex = prompt_mutex.clone();
        let thread_prompt_cv = prompt_cv.clone();
        let (message_tx, message_rx) = mpsc::channel();
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
                loop {
                    match message_rx.recv() {
                        Err(error) => {
                            println!("Receive Error in StdinLineStream readline loop thread: {:?}", error);
                        },
                        Ok(StdinLineStreamMessage::Write(message)) => {
                            io.write(message.as_bytes());
                        },
                        Ok(StdinLineStreamMessage::Prompt) => {
                            break;
                        }
                        Ok(StdinLineStreamMessage::Shutdown) => {
                            return;
                        }
                    }
                }
                match editor.readline(prompt, &mut io) {
                    Ok(line) => line_tx.send(line.to_owned()).unwrap(),
                    Err(_) => {},
                }
            }
        });
        Self {
            line_rx,
            message_tx
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

    pub fn prompt(&mut self) {
        self.message_tx.send(StdinLineStreamMessage::Prompt);
    }

    pub fn write(&mut self, message: String) {
        self.message_tx.send(StdinLineStreamMessage::Write(message));
    }
}

impl Drop for StdinLineStream {
    fn drop(&mut self) {
        _ = self.message_tx.send(StdinLineStreamMessage::Shutdown);
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
            exec_modes: [HartExecutionMode::Stopped; 4],
            breakpoints: Vec::new(),
            breakpoint_id_counter: 0,
        }
    }

    fn print_instruction(machine: &Arc<Machine>, address: u32, current: bool) {
        let opcode_value = match machine.read_u32(address) {
            ReadResult::Ok(value) => value,
            _ => 0
        };
        let opcode = hart::decoder::Rv32Op::decode(opcode_value);
        if current {
            println!("> {:08X}: {:08X} {}", address, opcode_value, opcode.assembly(address));
        } else {
            println!("  {:08X}: {:08X} {}", address, opcode_value, opcode.assembly(address));
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
            println!("  {symbol_string}");
            println!("")
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
        println!("rvfm debugger");
        let mut symbol_table: HashMap<String, u32> = HashMap::new();
        let mut symbol_tree: BTreeMap<u32, (String, symbol::Type)> = BTreeMap::new();
        let mut stdout = std::io::stdout();
        let mut line_stream = StdinLineStream::new();
        let mut last_execution_modes = [HartExecutionMode::Stopped; 4];
        let mut last_breakpoint_addrs = [3; 4];
        let mut hart_breakpoints = [0usize; 4];
        let mut current_hart = 0;
        let mut print_prompt = true;

        last_execution_modes[0] = HartExecutionMode::HitBreakpoint;
        self.exec_modes[0] = HartExecutionMode::HitBreakpoint;

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
                    line_stream.write(format!("failed to read elf file: {}", error.to_string()));
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
                            println!("hart {hart} hit breakpoint: {:08X}", pc);
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
                            println!("");
                            println!("hart {} stopped! pc: {:08X}", hart, self.harts[hart].pc);
                            println!("");
                            Self::print_code_region(&self.machine, self.harts[hart].pc, &symbol_tree);
                            print_prompt = true;
                            true
                        },
                        HartExecutionMode::Faulted => {
                            println!("");
                            println!("hart {} hit a fault!", hart);
                            println!("");
                            Self::print_code_region(&self.machine, self.harts[hart].pc, &symbol_tree);
                            print_prompt = true;
                            true
                        },
                        HartExecutionMode::HitBreakpoint => {
                            println!("");
                            println!("hart {} hit breakpoint!", hart);
                            println!("");
                            Self::print_code_region(&self.machine, self.harts[hart].pc, &symbol_tree);
                            print_prompt = true;
                            true
                        },
                        HartExecutionMode::WaitingForInterrupt => {
                            println!("");
                            println!("hart {} waiting for interrupt...", hart);
                            println!("");
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
                line_stream.prompt();
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
                        println!("added breakpoint {} at address {:08X}", id, addr);
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
                            println!("added breakpoint {} at {} ({:08X})", id, symbol_name, symbol_addr);
                        }
                    }
                    Some(Command::ClearBreakpoint(BreakpointName::Id(id))) => {
                        if let Some(index) = self.breakpoints.iter().position(|b| b.id == id) {
                            println!("removed breakpoint {}", id);
                            self.breakpoints.remove(index);
                        }
                    },
                    Some(Command::ClearBreakpoint(BreakpointName::Address(addr))) => {
                        if let Some(index) = self.breakpoints.iter().position(|b| b.address == addr) {
                            println!("removed breakpoint {}", self.breakpoints[index].id);
                            self.breakpoints.remove(index);
                        }
                    },
                    Some(Command::ClearBreakpoint(BreakpointName::Symbol(symbol_name))) => {
                        if let Some(symbol_addr) = symbol_table.get(&symbol_name) {
                            if let Some(index) = self.breakpoints.iter().position(|b| b.address == *symbol_addr) {
                                println!("removed breakpoint {}", self.breakpoints[index].id);
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
                                            Some(symbol_name) => println!("{}: {:08X} {}", breakpoint.id, breakpoint.address, symbol_name),
                                            None                       => println!("{}: {:08X}",    breakpoint.id, breakpoint.address),
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
                                    println!("hart {} stopped", hart);
                                },
                                (true, HartExecutionMode::Stopped) => {
                                    println!("hart {} already stopped", hart);
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
                            println!("switched to hart {}", hart);
                            current_hart = hart;
                        } else {
                            println!("already on hart {}", hart);
                        }
                    },
                    Some(Command::Regs) => {
                        for r in 0..32 {
                            if r & 3 == 3 {
                                println!("{:<5} {:08X}", format!("{}:", Rv32Op::register_name(r)), self.harts[current_hart].gprs[r as usize]);
                            } else {
                                print!("{:<5} {:08X}  ", format!("{}:", Rv32Op::register_name(r)), self.harts[current_hart].gprs[r as usize]);
                            }
                        }
                        println!("pc:   {:08X}", self.harts[current_hart].pc);
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
                            _ => println!("failed to read from {:08X}", addr),
                        }
                    },
                    Some(Command::Read(DataType::U16, addr)) => {
                        match self.machine.read_u16(addr) {
                            ReadResult::Ok(value) => print!("{:04X}", value),
                            _ => println!("failed to read from {:08X}", addr),
                        }
                    },
                    Some(Command::Read(DataType::U32, addr)) => {
                        match self.machine.read_u32(addr) {
                            ReadResult::Ok(value) => print!("{:08X}", value),
                            _ => println!("failed to read from {:08X}", addr),
                        }
                    },
                    Some(other) => {
                        println!("unimplemented command: {:?}", other);
                    },
                    None => {
                        if line.len() > 0 {
                            println!("failed to parse command: {:?}", line);
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
                    println!("");
                },
                "r" | "running" => {
                    println!("");
                },
                topic => {
                    println!("unknown help topic: {}", topic);
                }
            }
        } else {
            println!("commands:");
            println!("    - help (topic)                 : print help information (optionally for a topic)");
            println!("          aliases                  : help h");
            println!("          help topics              :");
            println!("              * b | breakpoints");
            println!("              * r | running");
            println!("");
            println!("    - quit                         : quits rvfm");
            println!("          aliases                  : quit q");
            println!("");
            println!("    - kill                         : kill the machine, stopping execution");
            println!("          aliases                  : kill k");
            println!("");
            println!("    - list [<list name>]           : enumerate members of a list");
            println!("          aliases                  : list l");
            println!("          lists                    :");
            println!("               * breakpoints");
            println!("               * symbol");
            println!("");
            println!("    - continue <hart> ([, <hart>]) : continues the targeted hart(s)");
            println!("          aliases                  : continue c");
            println!("          harts                    :");
            println!("              * 0");
            println!("              * 1");
            println!("              * 2");
            println!("              * 3");
            println!("              * all");
            println!("");
            println!("    - stop <hart> [(, <hart>)]     : stops the targeted hart(s)");
            println!("          aliases                  : stop S");
            println!("          harts                    :");
            println!("              * 0");
            println!("              * 1");
            println!("              * 2");
            println!("              * 3");
            println!("              * all");
            println!("");
            println!("    - hart <hart>                  : stops the targeted hart(s)");
            println!("          aliases                  : hart H");
            println!("          harts                    :");
            println!("              * 0");
            println!("              * 1");
            println!("              * 2");
            println!("              * 3");
            println!("");
            println!("    - set_bp <breakpoint>          : creates a breakpoint at the address/symbol specified");
            println!("          aliases                  : set_bp b");
            println!("          breakpoints              :");
            println!("              * <symbol name>");
            println!("              * @ <hex address>");
            println!("");
            println!("    - clear_bp <breakpoint>        : creates a breakpoint at the address/symbol specified");
            println!("          aliases                  : clear_bp !b");
            println!("          breakpoints              :");
            println!("              * <symbol name>");
            println!("              * @ <hex address>");
            println!("              * <breakpoint id>");
            println!("");
            println!("    - step                         : single-steps the current hart");
            println!("          aliases                  : step s");
            println!("");
            println!("    - regs                         : prints registers for the current hart");
            println!("          aliases                  : regs r");
        }
    }
}
