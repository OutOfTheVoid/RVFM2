use super::shader::*;
use super::texture::TextureModule;
use super::buffer::BufferModule;
use super::types::PixelDataLayout;

#[derive(Debug, Default)]
pub struct FragmentState {
    pub output_assignments: Vec<FragmentOutputAssignment>,
    pub depth_state: Option<FragmentDepthState>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FragmentOutputType {
    F32ToF32,
    F32ToInt,
    F32ToUInt,
    F32ToINorm,
    F32ToUNorm,
    IntToInt,
    IntToF32,
    UIntToUInt,
    UIntToF32,
}

impl FragmentOutputType {
    pub fn from_u8(x: u8) -> Option<Self> {
        Some(match x {
            0 => Self::F32ToF32,
            1 => Self::F32ToInt,
            2 => Self::F32ToUInt,
            3 => Self::F32ToINorm,
            4 => Self::F32ToUNorm,
            5 => Self::IntToInt,
            6 => Self::IntToF32,
            7 => Self::UIntToUInt,
            8 => Self::UIntToF32,
            _ => None?
        })
    }
}

pub const FRAGMENT_VECTOR_INPUT_BUILTIN_POSITION    : usize = 0x00;
pub const FRAGMENT_VECTOR_INPUT_BUILTIN_BARYCENTRIC : usize = 0x01;
pub const FRAGMENT_VECTOR_INPUT_USER_OFFSET         : usize = 0x10;

pub const FRAGMENT_SCALAR_OUTPUT_BUILTIN_DISCARD : usize = 0x00;
pub const FRAGMENT_SCALAR_OUTPUT_BUILTIN_DEPTH   : usize = 0x01;
pub const FRAGMENT_SCALAR_OUTPUT_USER_OFFSET     : usize = 0x10;

#[derive(Debug, Copy, Clone)]
pub struct FragmentOutputAssignment {
    pub output: u8,
    pub texture: u8,
    pub t: FragmentOutputType,
    pub c: ShaderCardinality,
    pub offset: [u32; 2]
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DepthCompareFn {
    Never,
    Always,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

impl DepthCompareFn {
    pub fn from_u8(x: u8) -> Option<Self> {
        Some(match x {
            0 => Self::Never,
            1 => Self::Always,
            2 => Self::Less,
            3 => Self::LessOrEqual,
            4 => Self::Greater,
            5 => Self::GreaterOrEqual,
            _ => None?
        })
    }
}

#[derive(Clone, Debug)]
pub struct FragmentDepthState {
    pub depth_texture: u8,
    pub compare_fn: DepthCompareFn,
    pub depth_write: bool,
}

pub struct FragmentShaderCall<'a> {
    pub state  : &'a FragmentState,
    pub shader : u8,

    pub fragmen_count: usize,

    pub shading_unit_context     : &'a mut ShadingUnitContext,
    pub shading_unit_run_context : &'a mut ShadingUnitRunContext<'a>,
    
    pub buffer_modules  : &'a mut [BufferModule; 256],
    pub texture_modules : &'a mut [TextureModule; 64],
    pub shader_modules  : &'a [ShaderModule; 128],

    pub resource_map: &'a ResourceMap,
}

pub fn run_fragment_shader(mut call: FragmentShaderCall<'_>) {
    let invocation_count = call.fragmen_count;
    let mut continue_instructions = true;

    let mut shader = &call.shader_modules[call.shader as usize];
    if shader.shader_type != ShaderType::Fragment {
        return;
    }
    let mut instructions = shader.instruction_buffer[0..shader.instruction_count].iter();
    while let (Some(instruction), true) = (instructions.next(), continue_instructions) {
        if call.shading_unit_context.run_instruction(invocation_count, instruction, &mut call.shading_unit_run_context, &mut call.buffer_modules, &mut call.texture_modules, call.resource_map).is_none() {
            continue_instructions = false;
        }
    }
    
    let mut depth_pass_buffer = [0u32; (CORE_COUNT + 31) >> 5];

    if let Some(depth_state) = &call.state.depth_state {
        let depth_texture_index = call.resource_map.texture[depth_state.depth_texture as usize] as usize;
        let depth_texture = &mut call.texture_modules[depth_texture_index];
        if depth_texture.config.pixel_layout != PixelDataLayout::D32x1 {
            return;
        }
        for f in 0..call.fragmen_count {
            let position = call.shading_unit_run_context.vector_input_array[FRAGMENT_VECTOR_INPUT_BUILTIN_POSITION][f].map(|x| f32::from_bits(x));
            let depth_val = position[2].clamp(0.0, 1.0);
            let depth_texture_val = f32::from_bits(depth_texture.fetch::<u32>(position[0] as u32, position[1] as u32));
            let pass = match depth_state.compare_fn {
                DepthCompareFn::Always         => true,
                DepthCompareFn::Never          => false,
                DepthCompareFn::Greater        => depth_val > depth_texture_val,
                DepthCompareFn::GreaterOrEqual => depth_val >= depth_texture_val,
                DepthCompareFn::Less           => depth_val < depth_texture_val,
                DepthCompareFn::LessOrEqual    => depth_val <= depth_texture_val,
            };
            if pass {
                if depth_state.depth_write {
                    depth_texture.store::<u32>(position[0] as u32, position[1] as u32, depth_val.to_bits());
                }
                depth_pass_buffer[f >> 5] |= (1 << (f & 31));
            }
        }
    } else {
        depth_pass_buffer.fill(0xFFFFFFFF);
    }
    'output: for output in call.state.output_assignments.iter() {
        let texture_index = call.resource_map.texture[output.texture as usize] as usize;
        let texture = &mut call.texture_modules[texture_index];
        match output.c {
            ShaderCardinality::Scalar => {
                let write_fn: fn(&mut TextureModule, u32, u32, u32) -> () = match (output.t, texture.config.pixel_layout) {

                    (FragmentOutputType::F32ToF32,   PixelDataLayout::D32x1) =>
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u32>(x, y, scalar_value)
                        },

                    (FragmentOutputType::F32ToF32,   _                     ) =>
                        continue 'output,

                    (FragmentOutputType::F32ToInt,   PixelDataLayout::D8x1 ) =>
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u8>(x, y, (f32::from_bits(scalar_value) * std::i8::MAX as f32) as i8 as u8);
                        },

                    (FragmentOutputType::F32ToInt,   PixelDataLayout::D16x1) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u16>(x, y, (f32::from_bits(scalar_value) * std::i16::MAX as f32) as i16 as u16);
                        },

                    (FragmentOutputType::F32ToInt,   PixelDataLayout::D32x1) =>
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u32>(x, y, (f32::from_bits(scalar_value) * std::i32::MAX as f32) as i32 as u32);
                        },

                    (FragmentOutputType::F32ToUInt,  PixelDataLayout::D8x1 ) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u8>(x, y, (f32::from_bits(scalar_value) * std::u8::MAX as f32) as u8);
                        },
                        
                    (FragmentOutputType::F32ToUInt,  PixelDataLayout::D16x1) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u16>(x, y, (f32::from_bits(scalar_value) * std::u16::MAX as f32) as u16);
                        },

                    (FragmentOutputType::F32ToUInt,  PixelDataLayout::D32x1) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u32>(x, y, (f32::from_bits(scalar_value) * std::u32::MAX as f32) as u32);
                        },
                        
                    (FragmentOutputType::F32ToINorm, PixelDataLayout::D8x1 ) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u8>(x, y, (f32::from_bits(scalar_value) * std::i8::MAX as f32) as i8 as u8);
                        },
                        
                    (FragmentOutputType::F32ToINorm, PixelDataLayout::D16x1) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u16>(x, y, (f32::from_bits(scalar_value) * std::i16::MAX as f32) as i16 as u16);
                        },

                    (FragmentOutputType::F32ToINorm, PixelDataLayout::D32x1) =>     
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u32>(x, y, (f32::from_bits(scalar_value) * std::i32::MAX as f32) as i32 as u32);
                        },

                    (FragmentOutputType::F32ToUNorm, PixelDataLayout::D8x1 ) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u8>(x, y, (f32::from_bits(scalar_value) * std::u8::MAX as f32) as u8);
                        },

                    (FragmentOutputType::F32ToUNorm, PixelDataLayout::D16x1) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u16>(x, y, (f32::from_bits(scalar_value) * std::u16::MAX as f32) as u16);
                        },

                    (FragmentOutputType::F32ToUNorm, PixelDataLayout::D32x1) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u32>(x, y, (f32::from_bits(scalar_value) * std::u32::MAX as f32) as u32);
                        },

                    (FragmentOutputType::IntToInt,   PixelDataLayout::D8x1 ) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u8>(x, y, scalar_value as i32 as i8 as u8);
                        },

                    (FragmentOutputType::IntToInt,   PixelDataLayout::D16x1) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u16>(x, y, scalar_value as i32 as i16 as u16);
                        },

                    (FragmentOutputType::IntToInt,   PixelDataLayout::D32x1) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u32>(x, y, scalar_value);
                        },

                    (FragmentOutputType::IntToF32,   PixelDataLayout::D32x1) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u32>(x, y, (scalar_value as i32 as f32).to_bits());
                        },

                    (FragmentOutputType::IntToF32,   _                     ) => continue 'output,

                    (FragmentOutputType::UIntToUInt, PixelDataLayout::D8x1 ) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u8>(x, y, scalar_value as u8);
                        },

                    (FragmentOutputType::UIntToUInt, PixelDataLayout::D16x1) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u16>(x, y, scalar_value as u16);
                        },

                    (FragmentOutputType::UIntToUInt, PixelDataLayout::D32x1) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u32>(x, y, scalar_value);
                        },

                    (FragmentOutputType::UIntToF32,  PixelDataLayout::D8x1 ) |
                    (FragmentOutputType::UIntToF32,  PixelDataLayout::D16x1) => continue 'output,

                    (FragmentOutputType::UIntToF32,  PixelDataLayout::D32x1) => 
                        |texture: &mut TextureModule, x: u32, y: u32, scalar_value: u32| {
                            texture.store::<u32>(x, y, (scalar_value as f32).to_bits());
                        },

                    (_                            ,  PixelDataLayout::D8x2 ) => continue 'output,
                    (_                            ,  PixelDataLayout::D8x4 ) => continue 'output,
                    (_                            ,  PixelDataLayout::D16x2) => continue 'output,
                    (_                            ,  PixelDataLayout::D16x4) => continue 'output,
                    (_                            ,  PixelDataLayout::D32x2) => continue 'output,
                    (_                            ,  PixelDataLayout::D32x4) => continue 'output,
                };
                let scalar_output = &call.shading_unit_run_context.scalar_output_array[output.output as usize];
                for f in 0..call.fragmen_count {
                    let position = call.shading_unit_run_context.vector_input_array[FRAGMENT_VECTOR_INPUT_BUILTIN_POSITION][f].map(|x| f32::from_bits(x) as u32);
                    write_fn(texture, output.offset[0] + position[0], output.offset[1] + position[1], scalar_output[f]);
                }
            },
            ShaderCardinality::V2 => {
                let texture_index = call.resource_map.texture[output.texture as usize] as usize;
                let texture = &mut call.texture_modules[texture_index];
                let write_fn: fn(&mut TextureModule, u32, u32, [u32; 4]) -> () = match (output.t, texture.config.pixel_layout) {
                    (FragmentOutputType::F32ToF32, PixelDataLayout::D32x2) => 
                        |texture: &mut TextureModule, x: u32, y: u32, vector_value: [u32; 4]| {
                            texture.store::<[u32; 2]>(x, y, [vector_value[0], vector_value[1]]);
                        },
                    _ => panic!("Unimplemented fragment output: {:?}", output),
                };
                let vector_output = &call.shading_unit_run_context.vector_output_array[output.output as usize];
                for f in 0..call.fragmen_count {
                    let position = call.shading_unit_run_context.vector_input_array[FRAGMENT_VECTOR_INPUT_BUILTIN_POSITION][f].map(|x| f32::from_bits(x) as u32);
                    write_fn(texture, output.offset[0] + position[0], output.offset[1] + position[1], vector_output[f]);
                }
            }
            ShaderCardinality::V3 => {
                let texture_index = call.resource_map.texture[output.texture as usize] as usize;
                let texture = &mut call.texture_modules[texture_index];
                let write_fn: fn(&mut TextureModule, u32, u32, [u32; 4]) -> () = match (output.t, texture.config.pixel_layout) {
                    (FragmentOutputType::F32ToF32, PixelDataLayout::D32x4) => 
                        |texture: &mut TextureModule, x: u32, y: u32, vector_value: [u32; 4]| {
                            texture.store::<[u32; 3]>(x, y, [vector_value[0], vector_value[1], vector_value[2]]);
                        },
                    _ => panic!("Unimplemented fragment output: {:?}", output),
                };
                let vector_output = &call.shading_unit_run_context.vector_output_array[output.output as usize];
                for f in 0..call.fragmen_count {
                    let position = call.shading_unit_run_context.vector_input_array[FRAGMENT_VECTOR_INPUT_BUILTIN_POSITION][f].map(|x| f32::from_bits(x) as u32);
                    write_fn(texture, output.offset[0] + position[0], output.offset[1] + position[1], vector_output[f]);
                }
            }
            ShaderCardinality::V4 => {
                let texture_index = call.resource_map.texture[output.texture as usize] as usize;
                let texture = &mut call.texture_modules[texture_index];
                let write_fn: fn(&mut TextureModule, u32, u32, [u32; 4]) -> () = match (output.t, texture.config.pixel_layout) {
                    (FragmentOutputType::F32ToF32, PixelDataLayout::D32x2) => 
                        |texture: &mut TextureModule, x: u32, y: u32, vector_value: [u32; 4]| {
                            texture.store::<[u32; 4]>(x, y, vector_value);
                        },
                    _ => panic!("Unimplemented fragment output: {:?}", output),
                };
                let vector_output = &call.shading_unit_run_context.vector_output_array[output.output as usize];
                for f in 0..call.fragmen_count {
                    let position = call.shading_unit_run_context.vector_input_array[FRAGMENT_VECTOR_INPUT_BUILTIN_POSITION][f].map(|x| f32::from_bits(x) as u32);
                    write_fn(texture, output.offset[0] + position[0], output.offset[1] + position[1], vector_output[f]);
                }
            }
        }
    }
}

