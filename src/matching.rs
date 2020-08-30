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
        threshold: f32,
    ) -> Vec<(&'func Function, f32)> {
        others
            .into_iter()
            .map(|other| (other, self.get_match_weight_for(input, other)))
            .filter(|(_, weight)| *weight >= threshold)
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
        let ingore_constant_data_section_pointers =
            self.options.ingore_constant_data_section_pointers;

        // TODO: Find a way to do this in a cleaner more modular way.
        match (left, right) {
            (Instruction::I32Store(_, left_offset), Instruction::I32Store(_, right_offset))
                if ingore_constant_data_section_pointers =>
            {
                self.both_locations_in_data_regions(*left_offset, *right_offset)
            }
            (Instruction::I32Const(left_const), Instruction::I32Const(right_const))
                if ingore_constant_data_section_pointers =>
            {
                let in_data_region =
                    self.both_locations_in_data_regions(*left_const as u32, *right_const as u32);

                in_data_region || left_const == right_const
            }
            (Instruction::CallIndirect(_, _), Instruction::CallIndirect(_, _)) => true,
            (Instruction::Call(_), Instruction::Call(_)) => true,
            (_, _) => left == right,
        }
    }

    fn does_signiture_match(&self, input: &Function, other: &Function) -> bool {
        let input = input;
        input.param_types == other.param_types && input.return_type == other.return_type
    }

    fn both_locations_in_data_regions(&self, left: u32, right: u32) -> bool {
        let left_inside = self
            .data_regions
            .iter()
            .any(|region| region.is_offset_inside(left));
        let right_inside = self
            .data_regions
            .iter()
            .any(|region| region.is_offset_inside(right));
        left_inside && right_inside
    }
}
