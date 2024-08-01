#![feature(new_uninit)]

use config::Config;
use hart::{StepState, Hart};
use machine::Machine;

mod machine;
mod debug;
mod hart;
mod gpu;
mod run;
mod debugger;
mod run_debugger;
mod ui;
mod hart_clock;
mod interrupt_controller;
mod spu;
mod config;
mod command_list;
mod pointer_queue;
mod input;

use run::{run, ROM_START_ADDRESS};
use run_debugger::run_debugger;
use ui::main_window;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config = Config::default();

    let mut debug_elf = None;

    for i in 1..args.len() - 1 {
        if args[i] == "-d" {
            debug_elf = Some(args[i + 1].clone());
        }
    }
    
    if args.len() == 1 {
        println!("usage: rvfm2 [flags] <rom>");
        println!("    flags:");
        println!("        * -d <rom elf>: Runs the debugger using the given elf file for ");
        println!("            debugging information. It is still necessary to pass the  ");
        println!("            rom binary.");
        println!("    rom:");
        println!("        The rom file to run as a flat binary. The base address and start");
        println!("        address of the rom is located at {:08X}", ROM_START_ADDRESS);
        return;
    }

    let rom_path = args[args.len() - 1].clone();
    let rom: Vec<u8> = std::fs::read(rom_path.clone()).expect(&format!("Failed to read rom file: {}", rom_path));
    
    ui::main_window::MainWindow::run(&config, move |main_window| {
        let (machine, machine_main_thread) = Machine::new(&rom[..], main_window.clone());
        drop(rom);
        if let Some(debug_elf) = debug_elf {
            run_debugger(machine.clone(), Some(&debug_elf));
        } else {
            run(&machine);
        }
        main_window.exit();
        drop(machine_main_thread);
    });
}
