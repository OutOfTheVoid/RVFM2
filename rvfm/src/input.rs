use std::{collections::HashMap, sync::Arc};
use crate::{machine::{ReadResult, WriteResult}, Machine};
use parking_lot::Mutex;
use winit::event::VirtualKeyCode;
use static_init::dynamic;

#[dynamic]
static INPUT_STATE: Mutex<HashMap<VirtualKeyCode, bool>> = Mutex::new(HashMap::new());
#[dynamic]
static INPUT_MAPPING: Mutex<HashMap<InputId, VirtualKeyCode>> = Mutex::new(HashMap::new());
#[dynamic]
static INPUT_LAST_PRESSED: Mutex<Option<VirtualKeyCode>> = Mutex::new(None);
#[dynamic]
static DEFAULT_INPUT_MAPPING: HashMap<InputId, VirtualKeyCode> = {
    let mut map = HashMap::new();
    map.insert(InputId::Up,    VirtualKeyCode::W);
    map.insert(InputId::Down,  VirtualKeyCode::S);
    map.insert(InputId::Left,  VirtualKeyCode::A);
    map.insert(InputId::Right, VirtualKeyCode::D);
    map.insert(InputId::A, VirtualKeyCode::N);
    map.insert(InputId::B, VirtualKeyCode::M);
    map.insert(InputId::X, VirtualKeyCode::J);
    map.insert(InputId::Y, VirtualKeyCode::K);
    map.insert(InputId::Start, VirtualKeyCode::Escape);
    map.insert(InputId::Select, VirtualKeyCode::Return);
    map
};

#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub enum InputId {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    X,
    Y,
    Start,
    Select,
}

pub fn input_keyboard_event_handler(event: winit::event::KeyboardInput) {
    let mut input_state = INPUT_STATE.lock();
    if let Some(virtual_keycode) = event.virtual_keycode {
        let old_state = input_state.get(&virtual_keycode);
        let state = match event.state {
            winit::event::ElementState::Pressed => true,
            winit::event::ElementState::Released => false,
        };
        match (state, old_state) {
            (true, None) |
            (true, Some(false)) => {
                *INPUT_LAST_PRESSED.lock() = Some(virtual_keycode);
            },
            _ => {}
        }
        input_state.insert(virtual_keycode, state);
    }
}

pub fn input_read_u32(offset: u32) -> ReadResult<u32> {
    let button = match offset {
        0x00  => InputId::Up,
        0x04  => InputId::Down,
        0x08  => InputId::Left,
        0x0C => InputId::Right,
        0x10 => InputId::A,
        0x14 => InputId::B,
        0x18 => InputId::X,
        0x1C => InputId::Y,
        0x20 => InputId::Start,
        0x24 => InputId::Select,

        _  => return ReadResult::InvalidAddress,
    };

    let mut mapping = INPUT_MAPPING.lock();
    let key_code = if let Some(input_key_code) = mapping.get(&button) {
        *input_key_code
    } else {
        *DEFAULT_INPUT_MAPPING.get(&button).unwrap()
    };
    if let Some(state) = INPUT_STATE.lock().get(&key_code) {
        ReadResult::Ok(if *state { 1 } else { 0 })
    } else {
        ReadResult::Ok(0)
    }
}

pub fn input_read_u16(offset: u32) -> ReadResult<u16> {
    input_read_u32(offset).map(|x| x as u16)
}

pub fn input_read_u8(offset: u32) -> ReadResult<u8> {
    input_read_u32(offset).map(|x| x as u8)
}

pub fn input_write_u32(offset: u32, value: u32) -> WriteResult {
    match offset {
        0x00 | 0x04 | 0x08 | 0x0C |
        0x10 | 0x14 | 0x18 | 0x1C |
        0x20 | 0x24 => WriteResult::Ok,
        0x28 => {
            *INPUT_LAST_PRESSED.lock() = None;
            WriteResult::Ok
        },
        _ => WriteResult::InvalidAddress
    }
}

pub fn input_write_u16(offset: u32, value: u16) -> WriteResult {
    input_write_u32(offset, value as u32)
}

pub fn input_write_u8(offset: u32, value: u8) -> WriteResult {
    input_write_u32(offset, value as u32)
}
