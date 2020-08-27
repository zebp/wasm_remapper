use crate::{
    parse::{DataRegion, Function},
    Remapper,
};
use parity_wasm::elements::Instruction;

pub struct MatchingContext<'a, 'wasm> {
    data_regions: &'a [DataRegion],
    options: &'a Remapper<'wasm>,
}

impl<'a, 'wasm> MatchingContext<'a, 'wasm> {
    pub fn new(data_regions: &'a [DataRegion], options: &'a Remapper<'wasm>) -> Self {
        Self {
            data_regions,
            options,
        }
    }

    pub fn find_matches<'func>(
        &self,
        input: &Function,
        others: &'func [Function],
    ) -> Vec<(&'func Function, f32)> {
        others
            .into_iter()
            .map(|other| (other, self.get_match_weight_for(input, other)))
            .collect()
    }

    fn get_match_weight_for(&self, input: &Function, other: &Function) -> f32 {
        let input_instructions = &input.instructions;
        let other_instructions = &other.instructions;
        let max_instructions = input_instructions.len().max(other_instructions.len());

        if !self.does_signiture_match(input, other) {
            return 0.0;
        } else if self.options.require_exact_function_locals
            && input.local_types != other.local_types
        {
            return 0.0;
        }

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

    fn does_signiture_match(&self, input: &Function, other: &Function) -> bool {
        let input = input;
        input.param_types == other.param_types && input.return_type == other.return_type
    }
}
