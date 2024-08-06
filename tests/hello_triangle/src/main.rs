#![no_std]
#![no_main]

use core::arch::global_asm;
use core::ptr::addr_of_mut;

global_asm!(include_str!("init.s"));

use rvfm_platform::gpu::*;
use rvfm_platform::command_list::*;
use rvfm_platform::intrin::*;
use rvfm_platform::debug::*;

use glam::{Mat4, Vec3, Quat};

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
    1.0,  0.0,  1.0,

    -0.5,  0.5,  0.0,
    1.0,  1.0,  0.0,

    0.5,  0.5,  0.0,
    0.0,  1.0,  1.0,
];

#[repr(C, align(4))]
pub struct ConstantData {
    pub numeric_constants: [f32; 4],
    pub transform_matrix: [f32; 16],
}

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

const RASTERIZER_STATE: RasterizerState = RasterizerState::new(
    &[
        VaryingAssignment {
            t: VaryingType::F32x4,
            i: Interpolation::Barycentric,
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
        },
        ConstantAssignment {
            constant: 1,
            src_buffer: CONSTANT_BUFFER_ID,
            offset: 16,
            c: ShaderCardinality::V4,
            t: ShaderInputType::F32FromF32,
        },
        ConstantAssignment {
            constant: 2,
            src_buffer: CONSTANT_BUFFER_ID,
            offset: 32,
            c: ShaderCardinality::V4,
            t: ShaderInputType::F32FromF32,
        },
        ConstantAssignment {
            constant: 3,
            src_buffer: CONSTANT_BUFFER_ID,
            offset: 48,
            c: ShaderCardinality::V4,
            t: ShaderInputType::F32FromF32,
        },
        ConstantAssignment {
            constant: 4,
            src_buffer: CONSTANT_BUFFER_ID,
            offset: 64,
            c: ShaderCardinality::V4,
            t: ShaderInputType::F32FromF32,
        },
    ],
    &[0, 1],
    &[0]
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

fn build_commandlist<'a, 'c: 'a>(command_list_bytes: &'a mut [u8], constant_data: &ConstantData, present_completion: &'c mut u32) -> Result<(CommandList<'a, GpuCommands>, CommandListCompletion<'c>), ()> {
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

    *present_completion = 0;

    let (builder, present_completion) = CommandListBuilder::new(command_list_bytes)
        .set_video_mode(VideoResolution::R512x384, true, true, true)?
        .configure_buffer              (VERTEX_BUFFER_ID,         VERTEX_DATA.len() as u32 * core::mem::size_of::<f32>() as u32)?
        .upload_buffer                 (VERTEX_BUFFER_ID,         VERTEX_DATA.as_ptr() as *const u8)?
        .configure_buffer              (CONSTANT_BUFFER_ID,       core::mem::size_of::<ConstantData>() as u32)?
        .upload_buffer                 (CONSTANT_BUFFER_ID,       constant_data as *const _ as *const u8)?
        
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
    let mut angle = 0.0;
    let mut constant_data = ConstantData {
        numeric_constants: [0.0, 0.0, 0.5, 1.0],
        transform_matrix: Mat4::IDENTITY.to_cols_array()
    };
    loop {
        angle += 0.01;
        constant_data.transform_matrix = Mat4::from_rotation_z(angle).to_cols_array();
        {
            let (command_list, mut present_completion) = build_commandlist(&mut command_list_bytes[..], &constant_data, &mut present_completion_variable).unwrap();
            let mut submit_completion = gpu_submit(command_list, &mut completion_variable);
            submit_completion.wait();
            present_completion.wait();
        }
    }

    loop {
        wfi();
    }
}
