use std::collections::VecDeque;

use super::{buffer::BufferModule, fragment_shader::*, shader::*, texture::TextureModule, vertex_shader::*};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Interpolation {
    ProvokingVertexFlat,
    Linear,
    Smooth,
    Max,
    Min,
}

impl Interpolation {
    pub fn from_u8(x: u8) -> Option<Self> {
        Some(match x {
            0 => Interpolation::ProvokingVertexFlat,
            1 => Interpolation::Linear,
            2 => Interpolation::Smooth,
            3 => Interpolation::Max,
            4 => Interpolation::Min,
            _ => None?
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShaderVaryingType {
    F32   (Interpolation),
    F32x2 (Interpolation),
    F32x3 (Interpolation),
    F32x4 (Interpolation),
    I32   (Interpolation),
    I32x2 (Interpolation),
    I32x3 (Interpolation),
    I32x4 (Interpolation),
}

impl ShaderVaryingType {
    pub fn from_u8(x: u8, interpolation: Interpolation) -> Option<Self> {
        Some(match x {
            0 => Self::F32   (interpolation),
            1 => Self::F32x2 (interpolation),
            2 => Self::F32x3 (interpolation),
            3 => Self::F32x4 (interpolation),
            4 => Self::I32   (interpolation),
            5 => Self::I32x2 (interpolation),
            6 => Self::I32x3 (interpolation),
            7 => Self::I32x4 (interpolation),
            _ => None?
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RasterizerVaryingAssignment {
    pub slot: u8,
    pub t: ShaderVaryingType,
}

#[derive(Debug, Clone, Default)]
pub struct RasterizerState {
    pub varyings: Vec<RasterizerVaryingAssignment>,
    pub constants: Vec<ShaderConstantAssignment>,
    pub resource_map: ResourceMap,
}

pub struct RasterRect {
    pub upper_left  : (u32, u32),
    pub lower_right : (u32, u32),
}

pub struct RasterizerCall<'a> {
    pub constant_array           :  &'a mut ShadingUnitConstantArray,
    pub io_arrays                : &'a mut [ShadingUnitIOArray; 3],

    pub buffer_modules           :  &'a mut [BufferModule; 256],
    pub texture_modules          :  &'a mut [TextureModule; 64],
    pub shader_modules           :  &'a [ShaderModule; 128],

    pub vertex_count             : usize,

    pub shading_unit_context     : &'a mut ShadingUnitContext,

    pub state                    : &'a RasterizerState,

    pub vertex_shader            : u8,
    pub vertex_state             : &'a VertexState,

    pub fragment_shader          : u8,
    pub fragment_state           : &'a FragmentState,

    pub target_rect              : RasterRect,

    pub resource_map             : &'a ResourceMap,
}

const TILE_SIZE: u32 = 32;

pub fn run_rasterizer(mut call: RasterizerCall<'_>) {
    setup_shader_constants(call.constant_array, &call.state.constants[..], &call.state.resource_map, call.buffer_modules);

    let mut vertex_count = call.vertex_count;
    let mut vertex_offset = 0;

    while vertex_count > 0 {
        let vertex_invocation_count = vertex_count - (vertex_count % 3);
        {
            println!("RASTERIZER: shading {} verts", vertex_invocation_count);
            let vertex_run_context = unsafe { ShadingUnitRunContext {
                scalar_input_array:  &mut *(&mut call.io_arrays[0].scalar_array as *mut _),
                vector_input_array:  &mut *(&mut call.io_arrays[0].vector_array as *mut _),
                scalar_output_array: &mut *(&mut call.io_arrays[1].scalar_array as *mut _),
                vector_output_array: &mut *(&mut call.io_arrays[1].vector_array as *mut _),
                scalar_constant_array: &mut *(&mut call.constant_array.scalar_constant_array as *mut _),
                vector_constant_array: &mut *(&mut call.constant_array.vector_constant_array as *mut _),
            } };
            let vertex_call = unsafe { VertexShaderCall {
                shader: call.vertex_shader,
                state: & *(call.vertex_state as *const _),
                vertex_count: vertex_invocation_count,
                vertex_offset,
                shading_unit_context: &mut *(call.shading_unit_context as *mut _),
                shading_unit_run_context: vertex_run_context,
                buffer_modules: &mut *(call.buffer_modules as *mut _),
                texture_modules: &mut *(call.texture_modules as *mut _),
                resource_map: call.resource_map,
                shader_modules: & *(call.shader_modules as *const _),
            } };
            let shader_result = match run_vertex_shader(vertex_call) {
                Ok(result) => {
                    vertex_count = result.remaining_count;
                    vertex_offset = result.remaining_offset;
                },
                Err(e) => {
                    println!("ERROR: {:?}", e);
                    return
                },
            };
            
        }

        let target_rect_width = call.target_rect.lower_right.0 - call.target_rect.upper_left.0;
        let target_rect_height = call.target_rect.lower_right.1 - call.target_rect.upper_left.1;

        println!("RASTERIZER: target size: ({} x {})", target_rect_width, target_rect_height);

        let mut fragment_invocation_count = 0;

        for t in 0..vertex_invocation_count / 3 {
            let v0 = t * 3 + 0;
            let v1 = t * 3 + 1;
            let v2 = t * 3 + 2;

            let discard_a = call.io_arrays[1].scalar_array[VERTEX_SCALAR_OUTPUT_BUILTIN_VERTEX_DISCARD][v0];
            let discard_b = call.io_arrays[1].scalar_array[VERTEX_SCALAR_OUTPUT_BUILTIN_VERTEX_DISCARD][v1];
            let discard_c = call.io_arrays[1].scalar_array[VERTEX_SCALAR_OUTPUT_BUILTIN_VERTEX_DISCARD][v2];

            println!("RASTERIZER: verices: {:?}, discards: {:?}", &[v0, v1, v2], &[discard_a, discard_b, discard_c]);

            if (discard_a | discard_b | discard_c) != 0 {
                continue;
            }

            let p0 = call.io_arrays[1].vector_array[VERTEX_VECTOR_OUTPUT_BUILTIN_VERTEX_POSITION][v0].map(|x| f32::from_bits(x));
            let p1 = call.io_arrays[1].vector_array[VERTEX_VECTOR_OUTPUT_BUILTIN_VERTEX_POSITION][v1].map(|x| f32::from_bits(x));
            let p2 = call.io_arrays[1].vector_array[VERTEX_VECTOR_OUTPUT_BUILTIN_VERTEX_POSITION][v2].map(|x| f32::from_bits(x));

            let p0_sc = [
                ((p0[0] as f32) * 0.5 + 0.5) * target_rect_width  as f32 + call.target_rect.upper_left.0 as f32,
                ((p0[1] as f32) * 0.5 + 0.5) * target_rect_height  as f32 + call.target_rect.upper_left.1 as f32,
                p0[2],
                p0[3]
            ];
            let p1_sc = [
                ((p1[0] as f32) * 0.5 + 0.5) * target_rect_width  as f32 + call.target_rect.upper_left.0 as f32,
                ((p1[1] as f32) * 0.5 + 0.5) * target_rect_height  as f32 + call.target_rect.upper_left.1 as f32,
                p0[2],
                p0[3]
            ];
            let p2_sc = [
                ((p2[0] as f32) * 0.5 + 0.5) * target_rect_width  as f32 + call.target_rect.upper_left.0 as f32,
                ((p2[1] as f32) * 0.5 + 0.5) * target_rect_height  as f32 + call.target_rect.upper_left.1 as f32,
                p0[2],
                p0[3]
            ];

            let x_min = p0[0].min(p1[0]).min(p2[0]);
            let y_min = p0[1].min(p1[1]).min(p2[1]);
            let x_max = p0[0].max(p1[0]).max(p2[0]);
            let y_max = p0[1].max(p1[1]).max(p2[1]);

            if x_max < -1.0 || x_min > 1.0 || y_max < -1.0 || y_min > 1.0 {
                continue;
            }

            let x_min_target = ((x_min as f32) * 0.5 + 0.5) * target_rect_width  as f32 + call.target_rect.upper_left.0 as f32;
            let x_max_target = ((x_max as f32) * 0.5 + 0.5) * target_rect_width  as f32 + call.target_rect.upper_left.0 as f32;
            let y_min_target = ((y_min as f32) * 0.5 + 0.5) * target_rect_height as f32 + call.target_rect.upper_left.1 as f32;
            let y_max_target = ((y_max as f32) * 0.5 + 0.5) * target_rect_height as f32 + call.target_rect.upper_left.1 as f32;

            let x_min_clip = ((x_min_target as i32).max(0) as u32).clamp(call.target_rect.upper_left.0, call.target_rect.lower_right.0);
            let x_max_clip = ((x_max_target as i32).max(0) as u32).clamp(call.target_rect.upper_left.0, call.target_rect.lower_right.0);
            let y_min_clip = ((y_min_target as i32).max(0) as u32).clamp(call.target_rect.upper_left.1, call.target_rect.lower_right.1);
            let y_max_clip = ((y_max_target as i32).max(0) as u32).clamp(call.target_rect.upper_left.1, call.target_rect.lower_right.1);
            // TODO: Tiling access pattern

            for y in y_min_clip..=y_max_clip {
                let mut x_min = f32::MAX;
                let mut x_max = f32::MIN;
                let points = [p0_sc, p1_sc, p2_sc, p0_sc, p1_sc];
                for e in 0..3 {
                    let mut pa = points[e];
                    let mut pb = points[e + 1];
                    if pa[1] > pb[1] {
                        std::mem::swap(&mut pa, &mut pb);
                    }
                    if !((pa[1] <= y as f32) && (pb[1] >= y as f32)) {
                        continue;
                    }
                    let edge_dx = pb[0] - pa[0];
                    let edge_dy = pb[1] - pa[1];
                    let dy = y as f32 - pa[1];
                    let dx = (dy * edge_dx) / edge_dy;
                    let x = pa[0] + dx;
                    x_min = x_min.min(x);
                    x_max = x_max.max(x);
                }
                let x_min = x_min.ceil() as u32;
                let x_max = x_max.floor() as u32;
                for x in x_min..=x_max {
                    let mut areas = [0.0; 3];
                    for e in 0..3 {
                        let dx_pb = x as f32 - points[e + 1][0];
                        let dy_pb = y as f32 - points[e + 1][1];
                        let dx_pc = x as f32 - points[e + 2][0];
                        let dy_pc = y as f32 - points[e + 2][1];
                        areas[e] = (dx_pc  * dy_pb) - (dx_pb * dy_pc);
                    }
                    let area_sum = areas[0] + areas[1] + areas[2];
                    let b0 = areas[0] / area_sum;
                    let b1 = areas[1] / area_sum;
                    let b2 = areas[2] / area_sum;
                    let z = p0[2] * b0 + p1[2] * b1 + p2[2] * b2;
                    call.io_arrays[2].vector_array[FRAGMENT_VECTOR_INPUT_BUILTIN_POSITION   ][fragment_invocation_count] = [(x as f32).to_bits(), (y as f32).to_bits(), (z as f32).to_bits(), 0];
                    call.io_arrays[2].vector_array[FRAGMENT_VECTOR_INPUT_BUILTIN_BARYCENTRIC][fragment_invocation_count] = [b0.to_bits(), b1.to_bits(), b2.to_bits(), 0];
                    for varying in call.state.varyings.iter() {
                        match varying.t {
                            ShaderVaryingType::F32x3(Interpolation::ProvokingVertexFlat) |
                            ShaderVaryingType::I32x3(Interpolation::ProvokingVertexFlat) => {
                                call.io_arrays[2].vector_array[varying.slot as usize][fragment_invocation_count][0] = call.io_arrays[1].vector_array[varying.slot as usize][v0][0];
                                call.io_arrays[2].vector_array[varying.slot as usize][fragment_invocation_count][1] = call.io_arrays[1].vector_array[varying.slot as usize][v0][1];
                                call.io_arrays[2].vector_array[varying.slot as usize][fragment_invocation_count][2] = call.io_arrays[1].vector_array[varying.slot as usize][v0][2];
                            },
                            ShaderVaryingType::F32x4(Interpolation::ProvokingVertexFlat) |
                            ShaderVaryingType::I32x4(Interpolation::ProvokingVertexFlat) => {
                                call.io_arrays[2].vector_array[varying.slot as usize][fragment_invocation_count] = call.io_arrays[1].vector_array[varying.slot as usize][v0];
                            },
                            ShaderVaryingType::F32x4(Interpolation::Smooth) => {
                                let val_0 = call.io_arrays[1].vector_array[varying.slot as usize][v0].map(|x| f32::from_bits(x));
                                let val_1 = call.io_arrays[1].vector_array[varying.slot as usize][v1].map(|x| f32::from_bits(x));
                                let val_2 = call.io_arrays[1].vector_array[varying.slot as usize][v2].map(|x| f32::from_bits(x));
                                println!("val_0: {:?}, val_1: {:?}, val_2: {:?}", val_0, val_1, val_2);
                                call.io_arrays[2].vector_array[varying.slot as usize][fragment_invocation_count] = [0, 1, 2, 3].map(|c| (val_0[c] * b0 + val_1[c] * b1 + val_2[c] * b2).to_bits());
                            },
                            _ => println!("GPU ERROR: shader varying type {:?} unimplemented!", varying.t),
                        }
                    }
                    fragment_invocation_count += 1;

                    if fragment_invocation_count == CORE_COUNT {
                        println!("invoking fragment shader {CORE_COUNT} times");
                        fragment_invocation_count = 0;
                        invoke_fragment_shader(CORE_COUNT, &mut call);
                    }
                }
            }
        }
        if fragment_invocation_count > 0 {
            println!("invoking fragment shader {fragment_invocation_count} times");
            invoke_fragment_shader(fragment_invocation_count, &mut call);
        }
    }
}

fn invoke_fragment_shader(invocation_count: usize, call: &mut RasterizerCall<'_>) {
    let mut fragment_run_context = unsafe { ShadingUnitRunContext {
        scalar_input_array:  &mut *(&mut call.io_arrays[2].scalar_array as *mut _),
        vector_input_array:  &mut *(&mut call.io_arrays[2].vector_array as *mut _),
        scalar_output_array: &mut *(&mut call.io_arrays[0].scalar_array as *mut _),
        vector_output_array: &mut *(&mut call.io_arrays[0].vector_array as *mut _),
        scalar_constant_array: &mut *(&mut call.constant_array.scalar_constant_array as *mut _),
        vector_constant_array: &mut *(&mut call.constant_array.vector_constant_array as *mut _),
    } };
    println!("fragment shader index: {}", call.fragment_shader);
    let fragment_call = FragmentShaderCall {
        state: call.fragment_state,
        shader: call.fragment_shader,
        fragmen_count: invocation_count,
        shading_unit_context: call.shading_unit_context,
        shading_unit_run_context: &mut fragment_run_context,
        buffer_modules: call.buffer_modules,
        texture_modules: call.texture_modules,
        shader_modules: call.shader_modules,
        resource_map: call.resource_map,
    };
    run_fragment_shader(fragment_call);
}

