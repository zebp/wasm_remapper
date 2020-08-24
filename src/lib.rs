use thiserror::Error;

#[derive(Debug)]
pub struct RemapperOptions<'wasm> {
    /// The binary representation of the wasm that should be remapped.
    pub input: &'wasm [u8],
    /// A wasm binary with debug names included that can be used as a reference.
    pub reference: &'wasm [u8],
    /// If constants that appear to be pointers into a wasm's data section should
    /// be ignored when comparing if two instructions match.
    pub ingore_constant_data_section_pointers: bool,
    /// If two functions need to have the exact same locals for them to be considered
    /// a potential match.
    pub require_exact_function_locals: bool,
}

#[derive(Debug)]
pub struct RemapperOutput {
    /// A wasm binary with debug symbols added from the reference binary.
    output: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum RemapperError {}

pub fn remap<'wasm>(_options: &RemapperOptions<'wasm>) -> Result<RemapperOutput, RemapperError> {
    todo!()
}

#[cfg(test)]
mod tests {
    // TODO: Layout the api structure in the tests.
}
