#![no_std]
#![no_main]

use core::arch::global_asm;
use core::ptr::addr_of_mut;

global_asm!(include_str!("init.s"));

use rvfm::gpu::*;
use rvfm::command_list::*;
use rvfm::debug::*;
use rvfm::intrin::*;
use rvfm::input::*;

fn build_commandlist<'a, 'c: 'a>(command_list_bytes: &'a mut [u8], present_completion: &'c mut u32) -> Result<CommandList<'a, GpuCommands>, ()> {
    let texture_config = TextureConfig {
        width: 512,
        height: 384,
        image_layout: ImageDataLayout::Contiguous,
        pixel_layout: PixelDataLayout::D8x4,
    };
    let (builder, present_completion) = CommandListBuilder::new(command_list_bytes)
        .set_video_mode(VideoResolution::R512x384, true, true, true)?
        .configure_texture(0, &texture_config)?
        .set_constant_sampler_unorm8(0, [255, 0, 0, 255])?
        .clear_texture(0, 0)?
        .present_texture(0, &present_completion, false)?;
    Ok((builder.finish(), present_completion))
}

#[no_mangle]
extern "C" fn main() {
    let mut present_completion_variable = 0u32;
    let mut submit_completion_variable = 0u32;
    let mut command_list_bytes = [0u8; 1024];
    let (command_list, present_completion) = build_commandlist(&mut command_list_bytes[..], &mut present_completion_variable).unwrap();
    gpu_submit(command_list, &mut submit_completion_variable);

    loop {
        wfi();
    }
}
