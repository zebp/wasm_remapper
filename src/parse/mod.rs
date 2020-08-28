mod function;

use parity_wasm::elements::*;
use std::convert::TryFrom;
use thiserror::Error;

pub use function::Function;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("invalid instruction used for data offset")]
    InvalidOffsetInstruction,
    #[error("missing {0} section")]
    MissingSection(&'static str),
}

#[derive(Debug)]
pub struct ModuleInfo {
    pub functions: Vec<Function>,
    pub data_regions: Vec<DataRegion>,
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
        let type_section = value
            .type_section()
            .ok_or(ParseError::MissingSection("type"))?;
        let function_types = collect_function_types(&type_section);
        let functions = value
            .function_section()
            .ok_or(ParseError::MissingSection("function"))?
            .entries();
        let function_bodies = value
            .code_section()
            .ok_or(ParseError::MissingSection("code"))?
            .bodies();

        let name_section: NameMap = value
            .names_section()
            .and_then(|name_section| name_section.functions())
            .map(|function_name_section| function_name_section.names().clone())
            .unwrap_or(IndexMap::with_capacity(functions.len()));

        let import_count = value.import_count(ImportCountType::Function);

        let functions: Vec<Function> = functions
            .iter()
            .zip(function_bodies)
            .enumerate()
            .map(|(id, (declaration, body))| (id + import_count, (declaration, body)))
            .map(|(id, (declaration, body))| {
                let function_type = &function_types[declaration.type_ref() as usize];
                let local_types = collect_locals(&body);

                let name: Option<String> = name_section
                    .get(id as u32)
                    .as_ref()
                    .map(|name| name.to_string());

                Function {
                    id: id as u32,
                    name,
                    param_types: function_type.params().to_vec(),
                    return_type: function_type.return_type(),
                    local_types,
                    instructions: body.code().elements().to_vec(),
                }
            })
            .collect();

        Ok(Self {
            data_regions,
            functions,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DataRegion {
    start: u32,
    end: u32,
    data: Vec<u8>,
}

impl DataRegion {
    pub fn is_offset_inside(&self, offset: u32) -> bool {
        offset >= self.start && offset <= self.end
    }
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

fn collect_locals(body: &FuncBody) -> Vec<ValueType> {
    body.locals()
        .into_iter()
        .map(|locals| {
            std::iter::repeat(locals.value_type())
                .take(locals.count() as usize)
                .collect::<Vec<ValueType>>()
        })
        .flatten()
        .collect()
}

fn collect_function_types(type_sections: &TypeSection) -> Vec<FunctionType> {
    type_sections
        .types()
        .into_iter()
        .filter_map(|wasm_type| match wasm_type {
            Type::Function(func_type) => Some(func_type),
        })
        .map(|func_type| func_type.clone())
        .collect()
}
