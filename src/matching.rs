use crate::{
    parse::{DataRegion, Function},
    Remapper,
};
use parity_wasm::elements::Instruction;

pub struct MatchingContext<'a, 'wasm> {
    input: &'a Function,
    data_regions: &'a [DataRegion],
    options: &'a Remapper<'wasm>,
}

impl<'a, 'wasm> MatchingContext<'a, 'wasm> {
    pub fn new(
        input: &'a Function,
        data_regions: &'a [DataRegion],
        options: &'a Remapper<'wasm>,
    ) -> Self {
        Self {
            input,
            data_regions,
            options,
        }
    }

    pub fn get_match_weight_for(&self, other: &Function) -> f32 {
        let input_instructions = &self.input.instructions;
        let other_instructions = &other.instructions;
        let max_instructions = input_instructions.len().max(other_instructions.len());

        // TODO: Do some checking to see if functions are even a potential match

        let matching_instruction_count = input_instructions
            .into_iter()
            .zip(other_instructions.into_iter())
            .filter(|(left, right)| self.do_instructions_match(left, right))
            .count();
        let matching_percentage = matching_instruction_count as f32 / max_instructions as f32;

        matching_percentage
    }

    fn do_instructions_match(&self, left: &Instruction, right: &Instruction) -> bool {
        todo!()
    }
}
