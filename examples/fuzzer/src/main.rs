use std::panic;

use arbitrary::{Arbitrary, Unstructured};
use honggfuzz::fuzz;
use kframework::kore::{App, Id, Parser, Pattern, Sort, SymbolId};
use kframework_ffi::kllvm;
use kframework_ffi::kllvm::{MarshalError, Marshaller, VarHandler};

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

/// Hardcoded kore string for the initial configuration
const INIT_CONFIG: &str = r#"
Lbl'-LT-'generatedTop'-GT-'{}(
  Lbl'-LT-'T'-GT-'{}(
    Lbl'-LT-'k'-GT-'{}(
      kseq{}(inj{SortPgm{}, SortKItem{}}(
        Lblinit'Unds'fuzz{}(
            FIELD1:SortInt,
            FIELD2:SortInt
        )
      ),
      dotk{}())
    ),
    Lbl'-LT-'state'-GT-'{}(Lbl'Stop'Map{}())),
    Lbl'-LT-'generatedCounter'-GT-'{}(\dv{SortInt{}}("0"))
)"#;

impl VarHandler for FuzzInput {
    fn substitute(&mut self, name: &str, _sort: &Sort) -> Result<Pattern, MarshalError> {
        let int_sort = Sort::App {
            id: Id::new("SortInt".to_string()).unwrap(),
            args: vec![],
        };
        match name {
            "FIELD1" => Ok(Pattern::Dv {
                sort: int_sort,
                value: self.field1.to_string().into(),
            }),
            "FIELD2" => Ok(Pattern::Dv {
                sort: int_sort,
                value: self.field2.to_string().into(),
            }),
            _ => Err(MarshalError::UnknownVar(format!(
                "Unrecognized variable: {}",
                name
            ))),
        }
    }
}

fn main() {
    kllvm::init();

    // Free kllvm's memory when panicking.
    panic::set_hook(Box::new(|_| {
        kllvm::free_all_memory();
    }));

    let kore_pattern = Parser::new(INIT_CONFIG).unwrap().pattern().unwrap();

    let mut marshaller: Marshaller<FuzzInput> = Marshaller::new(None);

    loop {
        fuzz!(|seed: &[u8]| {
            let mut u = Unstructured::new(seed);
            let Ok(input) = FuzzInput::arbitrary(&mut u) else {
                panic!("Failed to generate input from seed");
            };

            marshaller.set_handler(input);

            // Build the initial config, execute it, retrieve the final config kore string
            let pattern = marshaller.marshal(&kore_pattern).unwrap();
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
                            .first()
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
