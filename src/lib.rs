mod matching;
mod parse;

use thiserror::Error;

#[derive(Debug)]
pub struct Remapper<'wasm> {
    input: &'wasm [u8],
    reference: &'wasm [u8],
    ingore_constant_data_section_pointers: bool,
    require_exact_function_locals: bool,
}

impl<'wasm> Remapper<'wasm> {
    pub fn builder() -> RemapperBuilder<'wasm> {
        RemapperBuilder {
            input: None,
            reference: None,
            ingore_constant_data_section_pointers: true,
            require_exact_function_locals: true,
        }
    }
}

#[derive(Debug)]
pub struct RemapperBuilder<'wasm> {
    input: Option<&'wasm [u8]>,
    reference: Option<&'wasm [u8]>,
    ingore_constant_data_section_pointers: bool,
    require_exact_function_locals: bool,
}

impl<'wasm> RemapperBuilder<'wasm> {
    /// The binary representation of the wasm that should be remapped.
    pub fn input(mut self, input: &'wasm [u8]) -> Self {
        self.input = Some(input);
        self
    }

    /// A wasm binary with debug names included that can be used as a reference.
    pub fn reference(mut self, reference: &'wasm [u8]) -> Self {
        self.reference = Some(reference);
        self
    }

    /// If constants that appear to be pointers into a wasm's data section should
    /// be ignored when comparing if two instructions match. Enabled by default.
    pub fn ingore_constant_data_section_pointers(mut self, enabled: bool) -> Self {
        self.ingore_constant_data_section_pointers = enabled;
        self
    }

    /// If two functions need to have the exact same locals for them to be considered
    /// a potential match. Enabled by default.
    pub fn require_exact_function_locals(mut self, enabled: bool) -> Self {
        self.require_exact_function_locals = enabled;
        self
    }

    pub fn remap(&self) -> Result<RemapperOutput, RemapperError> {
        todo!()
    }
}

#[derive(Debug)]
pub struct RemapperOutput {
    /// A wasm binary with debug symbols added from the reference binary.
    output: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum RemapperError {
    #[error("input wasm not a valid wasm binary")]
    InvalidInputBinary,
    #[error("reference wasm not a valid wasm binary")]
    InvalidReferenceBinary,
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs;

    fn read_wasm(name: &str) -> Vec<u8> {
        fs::read(format!("test-cases/{}.wasm", name)).expect("unable to open test wasm")
    }

    #[test]
    fn test_invalid_input() {
        let reference = read_wasm("empty");
        let result = Remapper::builder()
            .input(&[])
            .reference(&reference)
            .remap();

        match result {
            Err(RemapperError::InvalidInputBinary) => {},
            _ => panic!("unexpected result")
        }
    }

    #[test]
    fn test_invalid_empty_reference() {
        let input = read_wasm("empty");
        let result = Remapper::builder()
            .input(&input)
            .reference(&[])
            .remap();

        match result {
            Err(RemapperError::InvalidReferenceBinary) => {},
            _ => panic!("unexpected result")
        }
    }
}
