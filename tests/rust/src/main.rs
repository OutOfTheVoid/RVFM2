#![no_std]
#![no_main]

use core::arch::global_asm;
use core::ptr::addr_of_mut;
use core::sync::atomic;

global_asm!(include_str!("init.s"));

use rvfm_platform::gpu::*;
use rvfm_platform::command_list::*;
use rvfm_platform::intrin::*;
use rvfm_platform::input::*;

fn build_commandlist<'a, 'c: 'a>(command_list_bytes: &'a mut [u8], present_completion: &mut StaticCommandListCompletion<'c>) -> Result<StaticCommandList<'a, GpuCommands>, GpuCommandBuilderError> {
    let texture_config = TextureConfig {
        width: 512,
        height: 384,
        image_layout: ImageDataLayout::Contiguous,
        pixel_layout: PixelDataLayout::D8x4,
    };
    let mut builder = StaticCommandListBuilder::new(command_list_bytes);
    builder.set_video_mode(VideoResolution::R512x384, true, true, true)?;
    builder.configure_texture(0, &texture_config)?;
    builder.set_constant_sampler_unorm8(0, [255, 0, 0, 255])?;
    builder.clear_texture(0, 0)?;
    builder.present_texture(0, present_completion, false)?;
    Ok(builder.finish())
}

#[no_mangle]
extern "C" fn main() {
    let submit_completion_variable = atomic::AtomicU32::new(0);
    let present_completion_variable = atomic::AtomicU32::new(0);
    let mut submit_completion = StaticCommandListCompletion::new(&submit_completion_variable);
    let mut present_completion = StaticCommandListCompletion::new(&present_completion_variable);
    let mut command_list_bytes = [0u8; 1024];
    let mut command_list = build_commandlist(&mut command_list_bytes[..], &mut present_completion).unwrap();
    gpu_submit(&mut command_list, &mut submit_completion);
    
    loop {
        wfi();
    }
}
