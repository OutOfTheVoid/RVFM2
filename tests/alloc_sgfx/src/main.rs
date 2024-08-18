#![no_std]
#![no_main]

use rvfm_platform::intrin::*;
use rvfm_platform::debug::*;
use rvfm_platform::multihart::spinlock::RawSpinLock;

use core::arch::global_asm;
use core::ptr::addr_of_mut;

global_asm!(include_str!("init.s"));

use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;

extern crate alloc;
use talc::*;

static mut ARENA: [u8; 10000] = [0; 10000];

#[global_allocator]
static ALLOCATOR: Talck<RawSpinLock, ErrOnOom> = Talc::new(ErrOnOom).lock();

#[no_mangle]
extern "C" fn main() {
    unsafe { ALLOCATOR.lock().claim(Span::from(addr_of_mut!(ARENA))).unwrap() };

    use rvfm_sgfx::{Instance, TextureConfig, PixelDataLayout, ImageDataLayout, VideoResolution};

    let mut instance = rvfm_sgfx::Instance::new();
    instance.set_video_mode(VideoResolution::R256x192);
    let texture_config = TextureConfig {
        pixel_layout: PixelDataLayout::D8x4,
        image_layout: ImageDataLayout::Contiguous,
        width: 256,
        height: 192,
    };
    let texture = instance.create_texture(&texture_config).unwrap();
    let constant_sampler = instance.create_constant_sampler().unwrap();
    let mut present_fence = instance.create_fence();
    let mut command_builder = instance.create_command_builder();
    command_builder.set_constant_sampler_unorm8(&constant_sampler, [0xFF, 0xFF, 0x00, 0xFF]);
    command_builder.clear_texture(&texture, &constant_sampler);
    command_builder.present_texture(&texture, &mut present_fence);
    let mut command_buffer = command_builder.build();
    instance.submit_command_buffer(&mut command_buffer);

    loop {
        wfi();
    }
}
