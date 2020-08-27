use parity_wasm::elements::{ValueType, Instruction};

#[derive(Debug)]
pub struct Function {
    pub id: u32,
    pub name: Option<String>,
    pub return_type: Option<ValueType>,
    pub param_types: Vec<ValueType>,
    pub local_types: Vec<ValueType>,
    pub instructions: Vec<Instruction>,
}