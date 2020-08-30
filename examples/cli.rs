use std::env::args;
use std::fs;
use wasm_remapper::Remapper;

fn main() {
    let args = args().collect::<Vec<_>>();
    let input_path = args.get(1).expect("invalid input path");
    let reference_path = args.get(2).expect("invalid reference path");
    let output_path = args.get(3).expect("invalid reference path");
    let threshold = args
        .get(4)
        .map(|threshold| {
            threshold
                .parse::<f32>()
                .expect("threshold not a floating number")
        })
        .unwrap_or(0.0);

    let input = fs::read(input_path).expect("unable to read input wasm");
    let reference = fs::read(reference_path).expect("unable to read input wasm");

    let remapper_output = Remapper::builder()
        .input(&input)
        .reference(&reference)
        .matching_threshold(threshold)
        .require_exact_function_locals(true)
        .build()
        .expect("could not create remapper")
        .remap()
        .expect("could not remap wasm");
    fs::write(output_path, remapper_output.output).expect("unable to write output binary");
    remapper_output
        .names
        .into_iter()
        .for_each(|(id, name)| println!("Remapped function {} to \"{}\"", id, name));
}
