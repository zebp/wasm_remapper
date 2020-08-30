mod matching;
mod parse;

use matching::MatchingContext;
use parity_wasm::elements::*;
use parse::ModuleInfo;
use std::convert::TryFrom;
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

    pub fn remap(&self) -> Result<RemapperOutput, RemapperError> {
        let input: Module = parity_wasm::deserialize_buffer(self.input)
            .map_err(|_| RemapperError::InvalidInputBinary)?;
        let reference: Module = parity_wasm::deserialize_buffer::<Module>(self.reference)
            .map_err(|_| RemapperError::InvalidReferenceBinary)?
            .parse_names()
            .map_err(|_| RemapperError::InvalidReferenceBinary)?;
        let input_info = ModuleInfo::try_from(&input)?;
        let reference_info = ModuleInfo::try_from(&reference)?;

        let mut data_regions = input_info.data_regions.clone();
        data_regions.extend(reference_info.data_regions.clone());

        let match_ctx = MatchingContext::new(&data_regions, &self);
        let name_map = self.build_name_map(&input_info, &reference_info, match_ctx);
        let name_section = self.build_name_section(&input, name_map.clone());

        let mut output_module = input.clone();
        output_module
            .insert_section(Section::Name(name_section))
            .expect("unable to insert custom name section into output module");
        let output_module_buf =
            parity_wasm::serialize(output_module).expect("unable to serialize output module");

        Ok(RemapperOutput {
            output: output_module_buf,
            names: name_map,
        })
    }

    fn build_name_map(
        &self,
        input_info: &ModuleInfo,
        reference_info: &ModuleInfo,
        match_ctx: MatchingContext,
    ) -> NameMap {
        let mut name_map = NameMap::with_capacity(input_info.functions.len());
        let mappings: Vec<_> = input_info
            .functions
            .iter()
            .map(|input_func| {
                let mut matches = match_ctx.find_matches(input_func, &reference_info.functions);
                matches.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
                (input_func, matches)
            })
            .filter(|(_, match_weights)| match_weights.len() > 0)
            .collect();

        for (function, match_weights) in mappings {
            let best_name = match_weights
                .first()
                .and_then(|(func, _)| func.name.clone());

            if let Some(best_name) = best_name {
                name_map.insert(function.id, best_name);
            }
        }

        name_map
    }

    fn build_name_section(&self, module: &Module, name_map: NameMap) -> NameSection {
        let mut buffer = Vec::new();
        name_map
            .serialize(&mut buffer)
            .expect("unable to build name section"); // This should never happen
        let name_subsection =
            FunctionNameSubsection::deserialize(module, &mut std::io::Cursor::new(buffer))
                .expect("unable to build name section");
        NameSection::new(None, Some(name_subsection), None)
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

    pub fn build(&self) -> Result<Remapper, RemapperError> {
        Ok(Remapper {
            input: self.input.ok_or(RemapperError::InvalidInputBinary)?,
            reference: self
                .reference
                .ok_or(RemapperError::InvalidReferenceBinary)?,
            ingore_constant_data_section_pointers: self.ingore_constant_data_section_pointers,
            require_exact_function_locals: self.require_exact_function_locals,
        })
    }
}

#[derive(Debug)]
pub struct RemapperOutput {
    /// A wasm binary with debug symbols added from the reference binary.
    pub output: Vec<u8>,
    /// A map of function ids to their new names in the output binary.
    pub names: NameMap,
}

#[derive(Debug, Error)]
pub enum RemapperError {
    #[error("input wasm not a valid wasm binary")]
    InvalidInputBinary,
    #[error("reference wasm not a valid wasm binary")]
    InvalidReferenceBinary,
    #[error("unable to parse {0}")]
    Parse(#[from] parse::ParseError),
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
        let reference = read_wasm("simple.reference");
        let result = Remapper::builder()
            .input(&[])
            .reference(&reference)
            .build()
            .unwrap()
            .remap();

        match result {
            Err(RemapperError::InvalidInputBinary) => {}
            _ => panic!("unexpected result"),
        }
    }

    #[test]
    fn test_invalid_empty_reference() {
        let input = read_wasm("simple.input");
        let result = Remapper::builder()
            .input(&input)
            .reference(&[])
            .build()
            .unwrap()
            .remap();

        match result {
            Err(RemapperError::InvalidReferenceBinary) => {}
            _ => panic!("unexpected result"),
        }
    }

    #[test]
    fn test_remap_simple() {
        let input = read_wasm("simple.input");
        let reference = read_wasm("simple.reference");
        let RemapperOutput { names, output } = Remapper::builder()
            .input(&input)
            .reference(&reference)
            .build()
            .unwrap()
            .remap()
            .unwrap();

        // Assert name map of the remapped output is valid
        assert_eq!(names.len(), 2);
        assert_eq!(names.get(0).unwrap(), "square");
        assert_eq!(names.get(1).unwrap(), "squareTen");

        let output_module = parity_wasm::deserialize_buffer::<Module>(&output)
            .expect("invalid output binary")
            .parse_names()
            .unwrap();
        let output_name_map = output_module
            .names_section()
            .expect("no name section in output module")
            .functions()
            .expect("no function name subsection")
            .names();

        // Ensure that the proveded output names match the actual names in the binary
        assert_eq!(&names, output_name_map);
    }
}
