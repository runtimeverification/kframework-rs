mod lexer;
mod parser;
mod serialize;
mod syntax;
mod visitor;

pub use parser::Parser;
pub use syntax::{
    App, Definition, Id, Module, Pattern, SVar, Sentence, SetVarId, Sort, Str, SymbolId, Var,
};
