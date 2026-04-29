use std::panic;

use arbitrary::{Arbitrary, Unstructured};
use honggfuzz::fuzz;
use kframework::kore::{App, Parser, Pattern, SymbolId};
use kframework_ffi::kllvm;

mod marshal;
#[allow(unused_imports)]
use marshal::{MarshalError, Marshaller, VarHandler};

#[derive(Clone, Copy)]
struct FuzzInput {
    field1: u32,
    field2: u32,
}

impl Arbitrary<'_> for FuzzInput {
    fn arbitrary(u: &mut Unstructured<'_>) -> arbitrary::Result<Self> {
        let field1 = u.int_in_range(0..=1000000)?;
        let field2 = u.int_in_range(0..=1000000)?;
        Ok(FuzzInput { field1, field2 })
    }
}

/// Hardcoded kore strings for assembling the initial configuration
const PREFIX: &str = r#"
Lbl'-LT-'generatedTop'-GT-'{}(
  Lbl'-LT-'T'-GT-'{}(
    Lbl'-LT-'k'-GT-'{}(
      kseq{}(inj{SortPgm{}, SortKItem{}}(
        Lblinit'Unds'fuzz{}(\dv{SortInt{}}(""#;
const MIDFIX: &str = r#""),\dv{SortInt{}}(""#;
const POSTFIX: &str = r#""))), dotk{}())), Lbl'-LT-'state'-GT-'{}(Lbl'Stop'Map{}())), Lbl'-LT-'generatedCounter'-GT-'{}(\dv{SortInt{}}("0")))"#;

impl From<FuzzInput> for String {
    /// Build the kore string to send off to kllvm's parser
    fn from(input: FuzzInput) -> String {
        let field1_str = input.field1.to_string();
        let field2_str = input.field2.to_string();

        format!(
            "{}{}{}{}{}",
            PREFIX, field1_str, MIDFIX, field2_str, POSTFIX
        )
    }
}

fn main() {
    kllvm::init();

    // Free kllvm's memory when panicking.
    panic::set_hook(Box::new(|_| {
        kllvm::free_all_memory();
    }));

    loop {
        fuzz!(|seed: &[u8]| {
            let mut u = Unstructured::new(seed);
            let Ok(input) = FuzzInput::arbitrary(&mut u) else {
                panic!("Failed to generate input from seed");
            };

            // Build the initial config, execute it, retrieve the final config kore string
            let pattern_string: String = input.clone().into();
            let pattern: kllvm::Pattern =
                pattern_string.parse().expect("Failed parsing kore string");
            let mut block: kllvm::Block = pattern.into();

            block.take_steps(-1);

            let result: kllvm::Pattern = block.into();

            // Parse the final kore string as [Pattern] for matching
            let result_pattern: Pattern = Parser::new(&result.to_string())
                .unwrap()
                .pattern()
                .expect("Failed to parse result pattern");

            match result_pattern {
                Pattern::App(App { symbol, args, .. }, ..) => {
                    if symbol == SymbolId::new("Lbl'-LT-'generatedTop'-GT-'".to_string()).unwrap() {
                        let expected = args
                            .get(0)
                            .expect("Expected first argument of generatedTop to be present");
                        match expected {
                            Pattern::App(App { symbol, .. }, ..) => {
                                if *symbol
                                    == SymbolId::new("Lblfuzz'Unds'failure".to_string()).unwrap()
                                {
                                    // <generatedTop> FUZZ_FAILURE </generatedTop>
                                    panic!("Failure!");
                                }
                            }
                            _ => panic!(
                                "Expected first argument of generatedTop to be an App pattern"
                            ),
                        }
                    } else {
                        panic!("Expected symbol for <generatedTop> but found {:?}", symbol);
                    }
                }
                _ => panic!("Expected App pattern but found: {:?}", result_pattern),
            };
        });
    }
}
