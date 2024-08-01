#![no_std]
#![no_main]

use core::arch::global_asm;
use core::ptr::addr_of_mut;

global_asm!(include_str!("init.s"));

use rvfm::gpu::*;
use rvfm::command_list::*;
use rvfm::intrin::*;
use rvfm::debug::*;

const VERTEX_SHADER_ID: u8 = 0;
const FRAGMENT_SHADER_ID: u8 = 1;
const RENDER_TEXTURE_ID: u8 = 0;
const VERTEX_BUFFER_ID: u8 = 0;
const CONSTANT_BUFFER_ID: u8 = 1;
const PIPELINE_STATE_ID: u8 = 0;
const CLEAR_CONSTANT_SAMPLER_ID: u8 = 0;

const VERTEX_SHADER   : &'static [u8] = include_bytes!("../bin/vshader.bin");
const FRAGMENT_SHADER : &'static [u8] = include_bytes!("../bin/fshader.bin");

const VERTEX_DATA: &'static [f32] = &[
    0.0, -0.5,  0.0,
    1.0,  0.0,  0.0,

    -0.5,  0.5,  0.0,
    0.0,  1.0,  0.0,

    0.5,  0.5,  0.0,
    0.0,  0.0,  1.0,
];

const CONSTANT_DATA: &'static [[f32; 4]] = &[
    [0.0, 1.0, 0.0, 1.0]
];

const RASTERIZER_STATE: RasterizerState = RasterizerState::new(
    &[
        VaryingAssignment {
            t: VaryingType::F32x4,
            i: Interpolation::Smooth,
            slot: 0x10,
            _dummy: 0,
        }
    ],
    &[
        ConstantAssignment {
            constant: 0,
            src_buffer: CONSTANT_BUFFER_ID,
            offset: 0,
            c: ShaderCardinality::V4,
            t: ShaderInputType::F32FromF32,
        }
    ],
    &[0, 1],
    &[0]
);

const VERTEX_STATE: VertexState = VertexState::new(
    &[
        VertexInputAssignment {
            input: 0x10,
            buffer_src: VERTEX_BUFFER_ID,
            input_type: ShaderInputType::F32FromF32,
            input_cardinality: ShaderCardinality::V3,
            offset: 0,
            stride: 24,
        },
        VertexInputAssignment {
            input: 0x11,
            buffer_src: VERTEX_BUFFER_ID,
            input_type: ShaderInputType::F32FromF32,
            input_cardinality: ShaderCardinality::V3,
            offset: 12,
            stride: 24,
        }
    ]
);

const FRAGMENT_STATE: FragmentState = FragmentState::new(None, &[
    FragmentOutputAssignment {
        output: 0x10,
        texture: RENDER_TEXTURE_ID,
        output_type: FragmentOutputType::F32ToUNorm,
        output_cardinality: ShaderCardinality::V4,
        offset: [0, 0],
    }
]);

const PIPELINE_STATE: GraphicsPipelineState = GraphicsPipelineState::new(
    &VERTEX_STATE,
    &FRAGMENT_STATE,
    &RASTERIZER_STATE
);

fn build_commandlist<'a, 'c: 'a>(command_list_bytes: &'a mut [u8], present_completion: &'c mut u32) -> Result<(CommandList<'a, GpuCommands>, CommandListCompletion<'c>), ()> {
    let texture_config = TextureConfig {
        width: 512,
        height: 384,
        image_layout: ImageDataLayout::Contiguous,
        pixel_layout: PixelDataLayout::D8x4,
    };

    let clipping_rect = ClippingRect {
        x_low: 0,
        y_low: 0,
        x_high: 512,
        y_high: 384,
    };

    let (builder, present_completion) = CommandListBuilder::new(command_list_bytes)
        .set_video_mode(VideoResolution::R512x384, true, true, true)?
        .configure_buffer              (VERTEX_BUFFER_ID,         VERTEX_DATA.len() as u32 * core::mem::size_of::<f32>() as u32)?
        .upload_buffer                 (VERTEX_BUFFER_ID,         VERTEX_DATA.as_ptr() as *const u8)?
        .configure_buffer              (CONSTANT_BUFFER_ID,       CONSTANT_DATA.len() as u32 * core::mem::size_of::<[f32; 4]>() as u32)?
        .upload_buffer                 (CONSTANT_BUFFER_ID,       CONSTANT_DATA.as_ptr() as *const u8)?
        
        .configure_texture             (RENDER_TEXTURE_ID,        &texture_config)?
        
        .upload_shader                 (VERTEX_SHADER_ID,         ShaderKind::Vertex, VERTEX_SHADER)?
        .upload_shader                 (FRAGMENT_SHADER_ID,       ShaderKind::Fragment, FRAGMENT_SHADER)?
        .upload_graphics_pipeline_state(PIPELINE_STATE_ID,         &PIPELINE_STATE)?
         
        .set_constant_sampler_unorm8   (CLEAR_CONSTANT_SAMPLER_ID, [0, 0, 0, 255])?
        .clear_texture                 (RENDER_TEXTURE_ID,         CLEAR_CONSTANT_SAMPLER_ID)?
        .draw_graphics_pipeline        (PIPELINE_STATE_ID,         VERTEX_SHADER_ID, FRAGMENT_SHADER_ID, 3, clipping_rect)?
        .present_texture               (RENDER_TEXTURE_ID,         present_completion, false)?;
    let command_list = builder.finish();
    Ok((command_list, present_completion))
}

#[no_mangle]
extern "C" fn main() {
    let mut completion_variable = 0u32;
    let mut present_completion_variable = 0u32;
    let mut command_list_bytes = [0u8; 1024];
    let (command_list, mut present_completion) = build_commandlist(&mut command_list_bytes[..], &mut present_completion_variable).unwrap();
    let mut submit_completion = gpu_submit(command_list, &mut completion_variable);

    println!("waiting for submit completion...");
    submit_completion.wait();
    println!("waiting for present completion...");
    present_completion.wait();
    println!("finished!");

    loop {
        wfi();
    }
}
