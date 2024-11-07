mod lexer;
mod parser;
mod syntax;

pub use parser::KoreParser;
pub use syntax::{
    App, Definition, Id, Module, Pattern, SVar, Sentence, SetVarId, Sort, Str, SymbolId, Var,
};
