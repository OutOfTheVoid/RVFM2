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

use run::run;
use run_debugger::run_debugger;
use ui::main_window;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let debug = args.iter().any(|x| x.contains("-d"));
    let config = Config::default();
    
    if args.len() == 1 {
        println!("usage: rvfm2 [-d] <rom>");
        return;
    }

    let rom_path = args[args.len() - 1].clone();
    let rom: Vec<u8> = std::fs::read(rom_path.clone()).expect(&format!("Failed to read rom file: {}", rom_path));
    
    ui::main_window::MainWindow::run(&config, move |main_window| {
        let (machine, machine_main_thread) = Machine::new(&rom[..], main_window.clone());
        drop(rom);
        if debug {
            run_debugger(machine.clone(), None);
        } else {
            run(&machine);
        }
        main_window.exit();
        drop(machine_main_thread);
    });
}
