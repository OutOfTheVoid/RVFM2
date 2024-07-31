use super::lexer::*;
use super::src_error::*;

trait WriteInstructionBytes {
    fn write(&self, bytes: &mut Vec<u8>, entry_type: EntryType);
}

const REGTYPE_LOCAL    : u8 = 0x00;
const REGTYPE_INPUT    : u8 = 0x01;
const REGTYPE_OUTPUT   : u8 = 0x02;
const REGTYPE_CONSTANT : u8 = 0x03;

impl WriteInstructionBytes for RegisterName {
    fn write(&self, bytes: &mut Vec<u8>, entry_type: EntryType) {
        let (regnum, regtype) = match (entry_type, self) {
            (_, RegisterName::LocalS(x)   ) => (*x, REGTYPE_LOCAL   ),
            (_, RegisterName::LocalV(x)   ) => (*x, REGTYPE_LOCAL   ),
            (_, RegisterName::ConstantS(x)) => (*x, REGTYPE_CONSTANT),
            (_, RegisterName::ConstantV(x)) => (*x, REGTYPE_CONSTANT),

            (_, RegisterName::InputS(x)   ) => (*x + 0x10, REGTYPE_INPUT   ),
            (_, RegisterName::InputV(x)   ) => (*x + 0x10, REGTYPE_INPUT   ),

            (_, RegisterName::OutputS(x)  ) => (*x + 0x10, REGTYPE_OUTPUT  ),
            (_, RegisterName::OutputV(x)  ) => (*x + 0x10, REGTYPE_OUTPUT  ),

            (EntryType::Vertex,   RegisterName::BuiltinS(ScalarBuiltin::VertexId       )) => (0x00, REGTYPE_INPUT ),
            (EntryType::Vertex,   RegisterName::BuiltinS(ScalarBuiltin::ProvokingVertex)) => (0x01, REGTYPE_INPUT ),

            (EntryType::Vertex,   RegisterName::BuiltinS(ScalarBuiltin::Discard        )) => (0x00, REGTYPE_OUTPUT),
            (EntryType::Vertex,   RegisterName::BuiltinV(VectorBuiltin::VertexPosition )) => (0x01, REGTYPE_OUTPUT),

            (EntryType::Fragment, RegisterName::BuiltinV(VectorBuiltin::VertexPosition )) => (0x00, REGTYPE_INPUT ),
            (EntryType::Fragment, RegisterName::BuiltinV(VectorBuiltin::Barycentric    )) => (0x01, REGTYPE_INPUT ),

            (EntryType::Fragment, RegisterName::BuiltinS(ScalarBuiltin::Discard        )) => (0x00, REGTYPE_OUTPUT),
            (EntryType::Fragment, RegisterName::BuiltinS(ScalarBuiltin::Depth          )) => (0x01, REGTYPE_OUTPUT),

            _ => panic!("Attempt to write illegal register address"),
        };
        bytes.push(regtype);
        bytes.push(regnum);
    }
}

const INSTRUCTION_VECTOR_PUSH                          : u8 = 0x00;
const INSTRUCTION_SCALAR_PUSH                          : u8 = 0x01;
const INSTRUCTION_VECTOR_POP                           : u8 = 0x02;
const INSTRUCTION_SCALAR_POP                           : u8 = 0x03;
const INSTRUCTION_VECTOR_COPY                          : u8 = 0x04;
const INSTRUCTION_SCALAR_COPY                          : u8 = 0x05;
const INSTRUCTION_VECTOR_COMPONENT_TO_SCALAR_COPY      : u8 = 0x06;
const INSTRUCTION_SCALAR_TO_VECTOR_COMPONENT_COPY      : u8 = 0x07;
const INSTRUCTION_COND_VECTOR_COPY                     : u8 = 0x04;
const INSTRUCTION_COND_SCALAR_COPY                     : u8 = 0x05;
const INSTRUCTION_COND_VECTOR_COMPONENT_TO_SCALAR_COPY : u8 = 0x06;
const INSTRUCTION_COND_SCALAR_TO_VECTOR_COMPONENT_COPY : u8 = 0x07;

pub fn write_push(bytes: &mut Vec<u8>, src: RegisterName, src_token: &Token, entry_type: EntryType) -> Result<(), SourceError> {
    bytes.push(if src.is_vector() { INSTRUCTION_VECTOR_PUSH } else { INSTRUCTION_SCALAR_PUSH });
    src.write(bytes, entry_type);
    Ok(())
}

pub fn write_pop(bytes: &mut Vec<u8>, dst: RegisterName, dst_token: &Token, entry_type: EntryType) -> Result<(), SourceError> {
    bytes.push(if dst.is_vector() { INSTRUCTION_VECTOR_POP } else { INSTRUCTION_SCALAR_POP });
    dst.write(bytes, entry_type);
    Ok(())
}

enum Component {
    X,
    Y,
    Z,
    W
}

impl Component {
    fn from_token(t: &Token) -> Result<Self, SourceError> {
        Ok(match t.value.as_ref().map(|s| s.as_str()).unwrap_or("") {
            "x" | "r" => Self::X,
            "y" | "g" => Self::Y,
            "z" | "b" => Self::Z,
            "w" | "a" => Self::W,
            _ => {
                Err(SourceError {
                    message: format!("Expected component name"),
                    line: t.line,
                    column: t.column,
                })?
            }
        })
    }
}

impl WriteInstructionBytes for Component {
    fn write(&self, bytes: &mut Vec<u8>, entry_type: EntryType) {
        bytes.push(match self {
            Self::X => 0x00,
            Self::Y => 0x01,
            Self::Z => 0x02,
            Self::W => 0x03,
        })
    }
}

pub fn write_mov(bytes: &mut Vec<u8>, mov_token: &Token, dst: RegisterName, dst_component: Option<&Token>, src: RegisterName, src_component: Option<&Token>, entry_type: EntryType) -> Result<(), SourceError> {
    Ok(match (dst.is_vector(), dst_component, src.is_vector(), src_component) {
        (true,  None,            true,  None           ) => {
            bytes.push(INSTRUCTION_VECTOR_COPY);
            dst.write(bytes, entry_type);
            src.write(bytes, entry_type);
        },

        (false, None,            false, None           ) => {
            bytes.push(INSTRUCTION_SCALAR_COPY);
            dst.write(bytes, entry_type);
            src.write(bytes, entry_type);
        },

        (false, None,            true,  Some(component)) => {
            let component = Component::from_token(component)?;
            bytes.push(INSTRUCTION_VECTOR_COMPONENT_TO_SCALAR_COPY);
            dst.write(bytes, entry_type);
            src.write(bytes, entry_type);
            component.write(bytes, entry_type);
        },

        (true,  Some(component), false, None           ) => {
            let component = Component::from_token(component)?;
            bytes.push(INSTRUCTION_SCALAR_TO_VECTOR_COMPONENT_COPY);
            dst.write(bytes, entry_type);
            component.write(bytes, entry_type);
            src.write(bytes, entry_type);
        },

        (false, None,            true,  None           ) |
        (true,  None,            false, None           ) => {
            Err(SourceError {
                message: format!("Type mismatch between dst and src"),
                line: mov_token.line,
                column: mov_token.column
            })?
        },

        (false, Some(component), _,     _              ) |
        (_,     _,               false, Some(component)) => {
            Err(SourceError {
                message: format!("Scalar registers don't have components"),
                line: component.line,
                column: component.column
            })?
        },

        (true,  Some(component), true,  _              ) |
        (true,  _,               true,  Some(component)) => {
            Err(SourceError {
                message: format!("No mov instruction exists for two vector components"),
                line: component.line,
                column: component.column
            })?
        }
    })
}


pub fn write_cmov(bytes: &mut Vec<u8>, mov_token: &Token, cond: RegisterName, dst: RegisterName, dst_component: Option<&Token>, src: RegisterName, src_component: Option<&Token>, entry_type: EntryType) -> Result<(), SourceError> {
    Ok(match (dst.is_vector(), dst_component, src.is_vector(), src_component) {
        (true,  None,            true,  None           ) => {
            bytes.push(INSTRUCTION_COND_VECTOR_COPY);
            cond.write(bytes, entry_type);
            dst.write(bytes, entry_type);
            src.write(bytes, entry_type);
        },

        (false, None,            false, None           ) => {
            bytes.push(INSTRUCTION_COND_SCALAR_COPY);
            cond.write(bytes, entry_type);
            dst.write(bytes, entry_type);
            src.write(bytes, entry_type);
        },

        (false, None,            true,  Some(component)) => {
            let component = Component::from_token(component)?;
            bytes.push(INSTRUCTION_COND_VECTOR_COMPONENT_TO_SCALAR_COPY);
            cond.write(bytes, entry_type);
            dst.write(bytes, entry_type);
            src.write(bytes, entry_type);
            component.write(bytes, entry_type);
        },

        (true,  Some(component), false, None           ) => {
            let component = Component::from_token(component)?;
            bytes.push(INSTRUCTION_COND_SCALAR_TO_VECTOR_COMPONENT_COPY);
            cond.write(bytes, entry_type);
            dst.write(bytes, entry_type);
            component.write(bytes, entry_type);
            src.write(bytes, entry_type);
        },

        (false, None,            true,  None           ) |
        (true,  None,            false, None           ) => {
            Err(SourceError {
                message: format!("Type mismatch between dst and src"),
                line: mov_token.line,
                column: mov_token.column
            })?
        },

        (false, Some(component), _,     _              ) |
        (_,     _,               false, Some(component)) => {
            Err(SourceError {
                message: format!("Scalar registers don't have components"),
                line: component.line,
                column: component.column
            })?
        },

        (true,  Some(component), true,  _              ) |
        (true,  _,               true,  Some(component)) => {
            Err(SourceError {
                message: format!("No cmov instruction exists for two vector components"),
                line: component.line,
                column: component.column
            })?
        }
    })
}
