
#[repr(C, align(4))]
#[derive(Copy, Clone, Debug)]
pub struct GraphicsPipelineState {
    pub vertex: *const VertexState,
    pub fragment: *const FragmentState,
    pub raster: *const RasterizerState,
}

impl GraphicsPipelineState {
    pub const fn new(vertex: &'static VertexState, fragment: &'static FragmentState, raster: &'static RasterizerState) -> Self {
        Self {
            vertex: vertex as *const _,
            fragment: fragment as *const _,
            raster: raster as *const _,
        }
    }
}

#[repr(C, align(4))]
#[derive(Copy, Clone, Debug)]
pub struct VertexState {
    pub inputs: *const VertexInputAssignment,
    pub input_count: u8,
    pub _dummy: [u8; 3]
}

impl VertexState {
    pub const fn new(inputs: &'static [VertexInputAssignment]) -> Self {
        Self {
            inputs: inputs.as_ptr(),
            input_count: inputs.len() as u8,
            _dummy: [0; 3]
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShaderInputType {
    UIntFromU8     = 0x00,
    UIntFromU16    = 0x01,
    UIntFromU32    = 0x02,
    IntFromI8      = 0x03,
    IntFromI16     = 0x04,
    IntFromI32     = 0x05,
    F32FromU8      = 0x06,
    F32FromU16     = 0x07,
    F32FromU32     = 0x08,
    F32FromI8      = 0x09,
    F32FromI16     = 0x0A,
    F32FromI32     = 0x0B,
    F32FromUNorm8  = 0x0C,
    F32FromUNorm16 = 0x0D,
    F32FromUNorm32 = 0x0E,
    F32FromINorm8  = 0x0F,
    F32FromINorm16 = 0x10,
    F32FromINorm32 = 0x11,
    F32FromF32     = 0x12,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShaderCardinality {
    Scalar = 0,
    V2     = 1,
    V3     = 2,
    V4     = 3
}

impl ShaderCardinality {
    pub fn from_u8(x: u8) -> Option<Self> {
        Some(match x {
            0 => Self::Scalar,
            1 => Self::V2,
            2 => Self::V3,
            3 => Self::V4,
            _ => None?
        })
    }
}

#[repr(C, align(4))]
#[derive(Copy, Clone, Debug)]
pub struct VertexInputAssignment {
    pub input: u8,
    pub buffer_src: u8,
    pub input_type: ShaderInputType,
    pub input_cardinality: ShaderCardinality,
    pub offset: u32,
    pub stride: u32,
}

#[repr(C, align(4))]
#[derive(Copy, Clone, Debug)]
pub struct FragmentState {
    pub depth_state: *const FragmentDepthState,
    pub output_assignments: *const FragmentOutputAssignment,
    pub output_assignment_count: u8,
    pub _dummy: [u8; 3],
}

impl FragmentState {
    pub const fn new(depth: Option<&'static FragmentDepthState>, output_assignments: &'static [FragmentOutputAssignment]) -> Self {
        Self {
            depth_state: if let Some(depth) = depth { depth as *const _ } else { core::ptr::null() },
            output_assignments: output_assignments.as_ptr(),
            output_assignment_count: output_assignments.len() as u8,
            _dummy: [0; 3]
        }
    }
}

#[repr(u8)]
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

#[repr(C, align(4))]
pub struct FragmentOutputAssignment {
    pub output: u8,
    pub texture: u8,
    pub output_type: FragmentOutputType,
    pub output_cardinality: ShaderCardinality,
    pub offset: [u32; 2],
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DepthCompareFn {
    Never,
    Always,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

#[repr(C, align(4))]
#[derive(Copy, Clone, Debug)]
pub struct FragmentDepthState {
    pub texture: u8,
    pub compare_fn: DepthCompareFn,
    pub depth_write: u8,
}

#[repr(C, align(4))]
#[derive(Copy, Clone, Debug)]
pub struct RasterizerState {
    pub varyings: *const VaryingAssignment,
    pub constants: *const ConstantAssignment,
    pub buffer_mappings: *const u8,
    pub texture_mappings: *const u8,
    pub varying_count: u8,
    pub constant_count: u8,
    pub buffer_mapping_count: u8,
    pub texture_mapping_count: u8,
}

impl RasterizerState {
    pub const fn new(varyings: &'static [VaryingAssignment], constants: &'static [ConstantAssignment], buffer_mappings: &'static [u8], texture_mappings: &'static[u8]) -> Self {
        Self {
            varyings: varyings.as_ptr(),
            varying_count: varyings.len() as u8,
            constants: constants.as_ptr(),
            constant_count: constants.len() as u8,
            buffer_mappings: buffer_mappings.as_ptr(),
            buffer_mapping_count: buffer_mappings.len() as u8,
            texture_mappings: texture_mappings.as_ptr(),
            texture_mapping_count: texture_mappings.len() as u8,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Interpolation {
    ProvokingVertex = 0,
    Linear = 1,
    Barycentric = 2,
    Max = 3,
    Min = 4,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VaryingType {
    F32   = 0,
    F32x2 = 1,
    F32x3 = 2,
    F32x4 = 3,
    I32   = 4,
    I32x2 = 5,
    I32x3 = 6,
    I32x4 = 7,
}

#[repr(C, align(4))]
#[derive(Copy, Clone, Debug)]
pub struct VaryingAssignment {
    pub t: VaryingType,
    pub i: Interpolation,
    pub slot: u8,
    pub _dummy: u8
}

#[repr(C, align(4))]
#[derive(Copy, Clone, Debug)]
pub struct ConstantAssignment {
    pub offset: u32,
    pub constant: u8,
    pub src_buffer: u8,
    pub c: ShaderCardinality,
    pub t: ShaderInputType,
}

