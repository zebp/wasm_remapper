use parity_wasm::elements::{DataSegment, Instruction, Module};
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("invalid instruction used for data offset")]
    InvalidOffsetInstruction,
}

#[derive(Debug)]
pub struct ModuleInfo {
    functions: Vec<()>,
    data_regions: Vec<DataRegion>,
}

impl TryFrom<&Module> for ModuleInfo {
    type Error = ParseError;

    fn try_from(value: &Module) -> Result<Self, Self::Error> {
        let data_regions: Vec<DataRegion> = value
            .data_section()
            .map(|data_section| {
                data_section
                    .entries()
                    .into_iter()
                    .map(DataRegion::try_from)
                    .collect()
            })
            .unwrap_or_else(|| Ok(Vec::new()))?;
        todo!()
    }
}

#[derive(Debug)]
pub struct DataRegion {
    start: u32,
    end: u32,
    data: Vec<u8>,
}

impl TryFrom<&DataSegment> for DataRegion {
    type Error = ParseError;

    fn try_from(value: &DataSegment) -> Result<Self, Self::Error> {
        let offset_instruction = value
            .offset()
            .as_ref()
            .ok_or(ParseError::InvalidOffsetInstruction)?
            .code()
            .first()
            .ok_or(ParseError::InvalidOffsetInstruction)?;
        let start = match offset_instruction {
            Instruction::I32Const(offset) => *offset as u32,
            _ => Err(ParseError::InvalidOffsetInstruction)?,
        };

        Ok(Self {
            start,
            end: start + value.value().len() as u32,
            data: value.value().to_vec(),
        })
    }
}
