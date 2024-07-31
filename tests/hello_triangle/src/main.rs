#![no_std]
#![no_main]

use core::arch::global_asm;
use core::ptr::addr_of_mut;

global_asm!(include_str!("init.s"));

use rvfm::gpu::*;
use rvfm::command_list::*;
use rvfm::intrin::*;

const VERTEX_SHADER   : &'static [u8] = include_bytes!("../bin/vshader.bin");
const FRAGMENT_SHADER : &'static [u8] = include_bytes!("../bin/fshader.bin");

const VERTEX_STATE: VertexState = VertexState::new(
    &[
        VertexInputAssignment {
            input: 0x10,
            buffer_src: 1,
            input_type: ShaderInputType::F32FromF32,
            input_cardinality: ShaderCardinality::V3,
            offset: 0,
            stride: 12,
        }
    ]
);

const FRAGMENT_STATE: FragmentState = FragmentState::new(None, &[
    FragmentOutputAssignment {
        output: 0x10,
        texture: 0,
        output_type: FragmentOutputType::F32ToUNorm,
        output_cardinality: ShaderCardinality::V4,
        offset: [0, 0],
    }
]);

const VERTEX_DATA: &'static [[f32; 3]] = &[
    [ 0.0, -0.5,  0.0],
    [-0.5,  0.5,  0.0],
    [ 0.5,  0.5,  0.0],
];

const CONSTANT_DATA: &'static [[f32; 4]] = &[
    [1.0, 0.0, 0.0, 1.0]
];

const RASTERIZER_STATE: RasterizerState = RasterizerState::new(
    &[
        VaryingAssignment {
            t: VaryingType::F32,
            i: Interpolation::ProvokingVertexFlat,
            slot: 0,
            _dummy: 0,
        }
    ],
    &[
        ConstantAssignment {
            constant: 0,
            src_buffer: 0,
            offset: 0,
            c: ShaderCardinality::V4,
            t: ShaderInputType::F32FromF32,
        }
    ],
    &[0, 1],
    &[0]
);

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
    let (builder, present_completion) = CommandListBuilder::new(command_list_bytes)
        .configure_buffer(1, VERTEX_DATA.len() as u32 * core::mem::size_of::<[f32; 3]>() as u32)?
        .upload_buffer(0, VERTEX_DATA as *const _ as *const u8)?
        .configure_buffer(0, CONSTANT_DATA.len() as u32 * core::mem::size_of::<[f32; 4]>() as u32)?
        .configure_texture(0, &texture_config)?
        .upload_shader(0, ShaderKind::Vertex, VERTEX_SHADER)?
        .upload_shader(1, ShaderKind::Fragment, FRAGMENT_SHADER)?
        .upload_graphics_pipeline_state(0, &PIPELINE_STATE)?
        .set_constant_sampler_f32(0, [0.0, 0.0, 0.0, 1.0])?
        .clear_texture(0, 0)?
        .present_texture(0, present_completion, false)?;
    let command_list = builder.finish();
    Ok((command_list, present_completion))
}

#[no_mangle]
extern "C" fn main() {
    let mut completion_variable = 0u32;
    let mut present_completion_variable = 0u32;
    let mut command_list_bytes = [0u8; 1024];
    let (command_list, present_completion) = build_commandlist(&mut command_list_bytes[..], &mut present_completion_variable).unwrap();
    gpu_submit(command_list, &mut completion_variable);

    loop {
        wfi();
    }
}
