#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ListType {
    Breakpoints,
    Symbols,
}

#[derive(Copy, Clone, Debug)]
pub enum DataType {
    Blob(u32),
    U32,
    U16,
    U8,
}

#[derive(Clone, Debug)]
pub enum BreakpointName {
    Symbol(String), // symbol_name
    Address(u32),   // @ <hex address>
    Id(usize),      // <number>
}

#[derive(Clone, Debug)]
pub enum Command {
    Help(Option<String>),
    Kill,
    Quit,
    List(Vec<ListType>),
    SetBreakpoint(BreakpointName),
    ClearBreakpoint(BreakpointName),
    Stop([bool; 4]),
    Continue([bool; 4]),
    Hart(usize),
    Regs,
    SingleStep,
    Read(DataType, u32),
    Write(DataType, u32, u32),
    //Frames,
}

fn decode_hex(hex: &str) -> u32 {
    let mut value_out: usize = 0;
    for char in hex.chars() {
        let value = match char {
            'a'..='f' => {
                Some(10 + char as usize - 'a' as usize)
            },
            'A'..='F' => {
                Some(10 + char as usize - 'A' as usize)
            },
            '0'..='9' => {
                Some(char as usize - '0' as usize)
            },
            _ => None
        };
        if let Some(value) = value {
            value_out <<= 4;
            value_out |= value;
        }
    }
    value_out as u32
}

impl Command {
    pub fn parse(command_string: &str) -> Option<Command> {
        if command_string.len() == 0 {
            return None;
        }
        let words: Vec<&str> = 
            command_string
                .trim()
                .split(|c: char| c.is_whitespace())
                .filter(|w| w.chars().nth(0).map(|x| !x.is_whitespace()).unwrap_or(true)).collect();
        match words.get(0) {
            Some(&"help") | Some(&"h") => {
                Some(Command::Help(words.get(1).map(|word| word.to_string())))
            },
            Some(&"kill") | Some(&"k") => {
                Some(Command::Kill)
            },
            Some(&"quit") | Some(&"q") => {
                Some(Command::Quit)
            },
            Some(&"list") | Some(&"l") => {
                let mut list_types = Vec::new();
                for list_type in words[1..].iter() {
                    match *list_type {
                        "breakpoints" => if !list_types.contains(&ListType::Breakpoints) { list_types.push(ListType::Breakpoints) },
                        "symbols" => list_types.push(ListType::Symbols),
                        _ => println!("unknown list: {}", list_type),
                    }
                }
                Some(Command::List(list_types))
            },
            Some(&"step") | Some(&"s") => Some(Command::SingleStep),
            Some(&"continue") | Some(&"c") => {
                let mut harts = [false; 4];
                for hart_target in words[1..].iter() {
                    match *hart_target {
                        "0" => harts[0] = true,
                        "1" => harts[1] = true,
                        "2" => harts[2] = true,
                        "3" => harts[3] = true,
                        "all" => harts = [true, true, true, true],
                        _ => return None
                    }
                }
                Some(Command::Continue(harts))
            },
            Some(&"stop") | Some(&"S") => {
                let mut harts = [false; 4];
                for hart_target in words[1..].iter() {
                    match *hart_target {
                        "0" => harts[0] = true,
                        "1" => harts[1] = true,
                        "2" => harts[2] = true,
                        "3" => harts[3] = true,
                        "all" => harts = [true, true, true, true],
                        _ => return None
                    }
                }
                Some(Command::Stop(harts))
            },
            Some(&"hart") | Some(&"H") => {
                if let Some(hart) = words.get(1) {
                    let hart = match *hart {
                        "0" => 0,
                        "1" => 1,
                        "2" => 2,
                        "3" => 3,
                        _ => return None
                    };
                    Some(Command::Hart(hart))
                } else {
                    None
                }
            },
            Some(&"regs") | Some(&"r") => {
                Some(Command::Regs)
            },
            Some(&"set_bp") | Some(&"b") => {
                match words[1..].first() {
                    Some(name_or_at) => {
                        if name_or_at == &"@" {
                            let breakpoint_address = match words[2..].first() {
                                Some(address_string) => {
                                    decode_hex(*&address_string)
                                },
                                None => return None,
                            };
                            Some(Command::SetBreakpoint(BreakpointName::Address(breakpoint_address)))
                        }else {
                            let symbol_name = match words[1..].first() {
                                Some(symbol_name) => symbol_name,
                                None => return None,
                            };
                            Some(Command::SetBreakpoint(BreakpointName::Symbol(symbol_name.to_string())))
                        }
                    },
                    None => return None
                }
            },
            Some(&"clear_bp") | Some(&"!b") => {
                match words[1..].first() {
                    Some(name_or_at) => {
                        if name_or_at == &"@" {
                            let breakpoint_address = match words[2..].first() {
                                Some(address_string) => {
                                    decode_hex(*&address_string)
                                },
                                None => return None,
                            };
                            Some(Command::ClearBreakpoint(BreakpointName::Address(breakpoint_address)))
                        } else if name_or_at.starts_with(&['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']) {
                            let mut id: usize = 0;
                            for char in name_or_at.chars() {
                                let value = match char {
                                    '0'..='9' => {
                                        Some(char as usize - '0' as usize)
                                    },
                                    _ => return None
                                };
                                if let Some(value) = value {
                                    id *= 10;
                                    id += value;
                                }
                            }
                            Some(Command::ClearBreakpoint(BreakpointName::Id(id)))
                        } else {
                            let symbol_name = match words[1..].first() {
                                Some(symbol_name) => symbol_name,
                                None => return None,
                            };
                            Some(Command::ClearBreakpoint(BreakpointName::Symbol(symbol_name.to_string())))
                        }
                    },
                    None => return None
                }
            },
            Some(&"read8") | Some(&"r8") => {
                let read_address = match words[1..].first() {
                    Some(address_string) => {
                        decode_hex(*&address_string)
                    },
                    None => return None,
                };
                Some(Command::Read(DataType::U8, read_address))
            },
            Some(&"read16") | Some(&"r16") => {
                let read_address = match words[1..].first() {
                    Some(address_string) => {
                        decode_hex(*&address_string)
                    },
                    None => return None,
                };
                Some(Command::Read(DataType::U16, read_address))
            },
            Some(&"read") | Some(&"read32") | Some(&"r") | Some(&"r32") => {
                let read_address = match words[1..].first() {
                    Some(address_string) => {
                        decode_hex(*&address_string)
                    },
                    None => return None,
                };
                Some(Command::Read(DataType::U32, read_address))
            },
            Some(&"write8") | Some(&"w8") => {
                let (address, value) = match words[1..] {
                    [address_string, value_string, ..] => {
                        (
                            decode_hex(address_string),
                            decode_hex(value_string)
                        )
                    },
                    _ => return None,
                };
                Some(Command::Write(DataType::U8, address, value))
            },
            Some(&"write16") | Some(&"w16") => {
                let (address, value) = match words[1..] {
                    [address_string, value_string, ..] => {
                        (
                            decode_hex(address_string),
                            decode_hex(value_string)
                        )
                    },
                    _ => return None,
                };
                Some(Command::Write(DataType::U16, address, value))
            },
            Some(&"write") | Some(&"w") => {
                let (address, value) = match words[1..] {
                    [address_string, value_string, ..] => {
                        (
                            decode_hex(address_string),
                            decode_hex(value_string)
                        )
                    },
                    _ => return None,
                };
                Some(Command::Write(DataType::U32, address, value))
            },
            _ => None,
        }
    }
}
