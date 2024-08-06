use std::sync::Arc;
use crate::machine::{Machine, ReadResult};
use super::rasterizer::{Interpolation, RasterizerState, RasterizerVaryingAssignment, ShaderVaryingType};
use super::shader::{ResourceMap, ShaderCardinality, ShaderInputType, ShaderConstantAssignment};
use super::vertex_shader::{VertexInputAssignment, VertexState};
use super::fragment_shader::{DepthCompareFn, FragmentDepthState, FragmentOutputAssignment, FragmentOutputType, FragmentState};

#[derive(Debug, Default)]
pub struct GraphicsPipelineState {
    pub vertex_state: VertexState,
    pub fragment_state: FragmentState,
    pub raster_state: RasterizerState,
}

impl GraphicsPipelineState {
    pub fn read_from_address(address: u32, machine: &Arc<Machine>) -> Option<Self> {
        let vertex_state_address = machine.read_u32(address).to_opt()?;
        let fragment_state_address = machine.read_u32(address + 4).to_opt()?;
        let rasterizer_state_address = machine.read_u32(address + 8).to_opt()?;
        let vertex_state = VertexState::read_from_address(vertex_state_address, machine)?;
        let fragment_state = FragmentState::read_from_address(fragment_state_address, machine)?;
        let raster_state = RasterizerState::read_from_address(rasterizer_state_address, machine)?;
        Some(
            GraphicsPipelineState {
                vertex_state,
                fragment_state,
                raster_state
            }
        )
    }
}

impl VertexState {
    fn read_from_address(address: u32, machine: &Arc<Machine>) -> Option<Self> {
        let input_array_address = machine.read_u32(address).to_opt()?;
        let input_count = machine.read_u8(address + 4).to_opt()?;
        let mut inputs = Vec::new();
        for i in 0..input_count {
            let input_address = input_array_address + i as u32 * 12;
            let input = machine.read_u8(input_address + 0).to_opt()?;
            let src_buffer = machine.read_u8(input_address + 1).to_opt()?;
            let input_type = machine.read_u8(input_address + 2).to_opt()?;
            let input_cardinality = machine.read_u8(input_address + 3).to_opt()?;
            let offset = machine.read_u32(input_address + 4).to_opt()?;
            let stride = machine.read_u32(input_address + 8).to_opt()?;
            let input_type = ShaderInputType::from_u8(input_type)?;
            let input_cardinality = ShaderCardinality::from_u8(input_cardinality)?;
            inputs.push(VertexInputAssignment {
                input,
                src_buffer,
                offset,
                stride,
                t: input_type,
                c: input_cardinality
            });
        }
        Some(VertexState {
            inputs
        })
    }
}

impl FragmentState {
    fn read_from_address(address: u32, machine: &Arc<Machine>) -> Option<Self> {
        let depth_state_address = machine.read_u32(address + 0).to_opt()?;
        let output_array_address = machine.read_u32(address + 4).to_opt()?;
        let output_count = machine.read_u8(address + 8).to_opt()?;
        let depth_state = if depth_state_address != 0 {
            let depth_texture = machine.read_u8(depth_state_address + 0).to_opt()?;
            let compare_fn = machine.read_u8(depth_state_address + 1).to_opt()?;
            let compare_fn = DepthCompareFn::from_u8(compare_fn)?;
            let depth_write = machine.read_u8(depth_state_address + 2).to_opt()?;
            Some(FragmentDepthState {
                depth_texture,
                compare_fn,
                depth_write: depth_write != 0
            })
        } else {
            None
        };
        let mut output_assignments = Vec::new();
        for o in 0..output_count {
            let output_assignment_address = output_array_address + 12 * o as u32;
            let output = machine.read_u8(output_assignment_address + 0).to_opt()?;
            let texture = machine.read_u8(output_assignment_address + 1).to_opt()?;
            let output_type = machine.read_u8(output_assignment_address + 2).to_opt()?;
            let output_type = FragmentOutputType::from_u8(output_type)?;
            let output_cardinality = machine.read_u8(output_assignment_address + 3).to_opt()?;
            let output_cardinality = ShaderCardinality::from_u8(output_cardinality)?;
            let offset_x = machine.read_u32(output_assignment_address + 4).to_opt()?;
            let offset_y = machine.read_u32(output_assignment_address + 8).to_opt()?;
            output_assignments.push(FragmentOutputAssignment {
                output,
                texture,
                t: output_type,
                c: output_cardinality,
                offset: [offset_x, offset_y]
            });
        }
        Some(FragmentState {
            depth_state,
            output_assignments
        })
    }
}

impl RasterizerState {
    fn read_from_address(address: u32, machine: &Arc<Machine>) -> Option<Self> {
        let varying_array_address = machine.read_u32(address + 0).to_opt()?;
        let constant_array_address = machine.read_u32(address + 4).to_opt()?;
        let buffer_mapping_array_address = machine.read_u32(address + 8).to_opt()?;
        let texture_mapping_array_address = machine.read_u32(address + 12).to_opt()?;
        let varying_count = machine.read_u8(address + 16).to_opt()?;
        let constant_count = machine.read_u8(address + 17).to_opt()?;
        println!("RasterizerState::read_from_address(): constant_count = {constant_count}, constant_array_address: {:08X}", constant_array_address);
        let buffer_mapping_count = machine.read_u8(address + 18).to_opt()?;
        let texture_mapping_count = machine.read_u8(address + 19).to_opt()?;
        let mut varyings = Vec::new();
        let mut constants = Vec::new();
        for v in 0..varying_count {
            let varying_address = varying_array_address + v as u32 * 4;
            let interpolation = machine.read_u8(varying_array_address + 1).to_opt()?;
            let interpolation = Interpolation::from_u8(interpolation)?;
            let varying_type = machine.read_u8(varying_array_address + 0).to_opt()?;
            let varying_type = ShaderVaryingType::from_u8(varying_type, interpolation)?;
            let slot = machine.read_u8(varying_array_address + 2).to_opt()?;
            varyings.push(RasterizerVaryingAssignment {
                slot,
                t: varying_type
            });
        }
        for c in 0..constant_count {
            let constant_address = constant_array_address + c as u32 * 8;
            let offset = machine.read_u32(constant_address + 0).to_opt()?;
            let constant = machine.read_u8(constant_address + 4).to_opt()?;
            let source_buffer = machine.read_u8(constant_address + 5).to_opt()?;
            let c = machine.read_u8(constant_address + 6).to_opt()?;
            let c = ShaderCardinality::from_u8(c)?;
            let t = machine.read_u8(constant_address + 7).to_opt()?;
            let t = ShaderInputType::from_u8(t)?;
            
            constants.push(ShaderConstantAssignment {
                constant,
                source_buffer,
                offset,
                t,
                c
            });
        }
        let mut resource_map = ResourceMap::default();
        if buffer_mapping_count != 0 {
            machine.read_block(buffer_mapping_array_address, &mut resource_map.buffer[0..buffer_mapping_count as usize]).to_opt()?;
        }
        if texture_mapping_count != 0 {
            machine.read_block(buffer_mapping_array_address, &mut resource_map.texture[0..texture_mapping_count as usize]).to_opt()?;
        }

        Some(RasterizerState {
            varyings,
            constants,
            resource_map
        })
    }
}
