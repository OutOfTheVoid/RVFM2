#![no_std]
#![no_main]

use core::arch::global_asm;
use core::ptr::addr_of_mut;

global_asm!(include_str!("init.s"));

use rvfm::gpu::*;
use rvfm::command_list::*;
use rvfm::debug::*;
use rvfm::intrin::*;
use rvfm::completion::Completion;
use rvfm::input::*;

static mut dummy_completion: u32 = 0;

fn build_commandlist<'a>(command_list_bytes: &'a mut [u8]) -> Result<CommandList<'a, GpuCommands>, ()> {
    let texture_config = TextureConfig {
        width: 512,
        height: 384,
        image_layout: ImageDataLayout::Contiguous,
        pixel_layout: PixelDataLayout::D8x4,
    };
    let present_completion = Completion::new(unsafe { addr_of_mut!(dummy_completion) });
    Ok(CommandListBuilder::new(command_list_bytes)
        .set_video_mode(VideoResolution::R512x384, true, true, true)?
        .configure_textre(0, &texture_config)?
        .set_constant_sampler_unorm8(0, [255, 0, 0, 255])?
        .clear_texture(0, 0)?
        .present_texture(0, &present_completion, false)?
        .finish())
}

#[no_mangle]
extern "C" fn main() {
    let submit_completion = Completion::new(unsafe { addr_of_mut!(dummy_completion) });
    let mut command_list_bytes = [0u8; 1024];
    let command_list = build_commandlist(&mut command_list_bytes[..]).unwrap();
    gpu_submit(command_list, &submit_completion);

    loop {
        wfi();
    }
}
