
use std::path::Path;
use std::sync::Arc;

use crate::hart::{StepState, Hart};
use crate::ui::main_window::{self, MainWindow};
use crate::{debugger::*, debug};
use crate::machine::Machine;

pub fn run_debugger(machine: Arc<Machine>, program_file: Option<&str>) {
    let mut debugger = Debugger::new(machine);
    debugger.run(program_file);
}
