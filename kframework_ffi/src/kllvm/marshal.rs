use super::{Pattern, Sort, Symbol};
use kframework::kore;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Debug)]
pub enum MarshalError {
    Unsupported(&'static str),
    UnknownVar(String),
    Cstring,
    InvalidString(String, usize),
}

pub trait VarHandler {
    fn substitute(&mut self, name: &str, sort: &kore::Sort) -> Result<kore::Pattern, MarshalError>;
}

/// Whether this marshalling is allowed to insert into the subtree cache.
/// `Disabled` propagates downward through Var-substituted subtrees, whose
/// Patterns live on the caller's stack and would leave dangling pointer
/// keys if cached.
#[derive(Clone, Copy)]
enum Caching {
    Allowed,
    Disabled,
}

/// Result of marshalling one node. Encodes ownership and var-freedom so
/// callers can determine whether or not to cache it. The lifetime ties
/// `Cached` to the marshaller.
enum Marshalled<'a> {
    Cached(&'a Pattern),
    Fresh { pattern: Pattern, var_free: bool },
}

/// A marshalling utility for moving a [`kore::Pattern`] over
/// to an llvm-backend's [`Pattern`]
///
/// Optionally uses a [`VarHandler`] to substitute variable terms
/// in the tree with concrete ones.
///
/// It caches any variable-free trees, keyed by the pointer to the
/// source tree.
///
/// This marshaller is good if:
/// - You have one tree that you want to marshal over once.
/// - You have a tree with variables that you want to marshal over multiple
///   times, but with different substitutions for the variables each time
///   (ie. you're running a fuzzer)
///
/// This marshaller is NOT good if:
/// - You are creating many different trees and want to marshal
///   over every one of them, and they contain few/no common
///   subtrees.
pub struct Marshaller<H: VarHandler> {
    symbols: HashMap<String, Symbol>,
    subtrees: HashMap<*const kore::Pattern, Pattern>,
    handler: Option<H>,
}

impl<H: VarHandler> Marshaller<H> {
    pub fn new(handler: Option<H>) -> Self {
        Self {
            symbols: HashMap::new(),
            subtrees: HashMap::new(),
            handler,
        }
    }

    /// Set the handler for any variable substitutions that need to be
    /// made. Replaces any pre-existing handler.
    pub fn set_handler(&mut self, h: H) {
        self.handler = Some(h);
    }

    /// Marshal over a [`kore::Pattern`] to the llvm-backend.
    ///
    /// Caches variable-free subtrees for repeated uses. The cache is
    /// keyed by `*const kore::Pattern`, so it is expected that
    /// structurally equivalent trees are actually the same tree.
    pub fn marshal(&mut self, root: &kore::Pattern) -> Result<Pattern, MarshalError> {
        // Drop the Marshalled<'_> (and any borrow it holds on self.subtrees)
        // before mutating the cache.
        match self.marshal_node(root, Caching::Allowed)? {
            Marshalled::Fresh { pattern, .. } => Ok(pattern),
            Marshalled::Cached(_) => Ok(self
                .subtrees
                .remove(&(root as *const kore::Pattern))
                .expect("Cached root must be present in the cache")),
        }
    }

    fn marshal_node(
        &mut self,
        p: &kore::Pattern,
        caching: Caching,
    ) -> Result<Marshalled<'_>, MarshalError> {
        let key = p as *const kore::Pattern;
        if matches!(caching, Caching::Allowed) && self.subtrees.contains_key(&key) {
            return Ok(Marshalled::Cached(self.subtrees.get(&key).unwrap()));
        }
        let (pattern, var_free) = match p {
            kore::Pattern::Var(v) => self.marshal_var(v)?,
            kore::Pattern::Dv { sort, value } => self.marshal_dv(sort, value)?,
            kore::Pattern::App(app) => self.marshal_app(app, caching)?,
            other => return Err(MarshalError::Unsupported(variant_name(other))),
        };
        Ok(self.maybe_cache(p, pattern, var_free, caching))
    }

    fn marshal_var(&mut self, v: &kore::Var) -> Result<(Pattern, bool), MarshalError> {
        let handler = self
            .handler
            .as_mut()
            .ok_or_else(|| MarshalError::UnknownVar(v.id.as_str().into()))?;
        let sub = handler.substitute(v.id.as_str(), &v.sort)?;
        let Marshalled::Fresh { pattern, .. } = self.marshal_node(&sub, Caching::Disabled)? else {
            unreachable!("Caching::Disabled should return Marshalled::Fresh")
        };
        Ok((pattern, false))
    }

    fn marshal_dv(
        &mut self,
        sort: &kore::Sort,
        value: &kore::Str,
    ) -> Result<(Pattern, bool), MarshalError> {
        let s = build_sort(sort)?;
        let pattern = Pattern::new_token(&value.0, &s)
            .map_err(|i| MarshalError::InvalidString(value.0.clone(), i))?;
        Ok((pattern, true))
    }

    fn marshal_app(
        &mut self,
        app: &kore::App,
        caching: Caching,
    ) -> Result<(Pattern, bool), MarshalError> {
        let mut pattern = {
            let sym = build_symbol(&app.symbol, &app.sorts)?;
            Pattern::from_symbol(&sym)
        };

        let mut var_free = true;
        for arg in &app.args {
            match self.marshal_node(arg, caching)? {
                Marshalled::Cached(child) => pattern.add_argument(child),
                Marshalled::Fresh {
                    pattern: child,
                    var_free: cvf,
                } => {
                    pattern.add_argument(&child);
                    var_free &= cvf;
                }
            }
        }

        Ok((pattern, var_free))
    }

    fn maybe_cache(
        &mut self,
        p: &kore::Pattern,
        pattern: Pattern,
        var_free: bool,
        caching: Caching,
    ) -> Marshalled<'_> {
        if matches!(caching, Caching::Allowed) && var_free {
            Marshalled::Cached(
                self.subtrees
                    .entry(p as *const kore::Pattern)
                    .or_insert(pattern),
            )
        } else {
            Marshalled::Fresh { pattern, var_free }
        }
    }

    #[allow(unused)]
    fn intern_symbol(
        &mut self,
        id: &kore::SymbolId,
        sorts: &[kore::Sort],
    ) -> Result<&Symbol, MarshalError> {
        let key = id.as_str().to_owned();
        Ok(match self.symbols.entry(key) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let mut sym = Symbol::new(id.as_str()).map_err(|_| MarshalError::Cstring)?;
                for sort in sorts {
                    let s = build_sort(sort)?;
                    sym.add_formal_argument(&s);
                }
                v.insert(sym)
            }
        })
    }
}

fn build_sort(s: &kore::Sort) -> Result<Sort, MarshalError> {
    match s {
        kore::Sort::App { id, args } => {
            let mut sort = Sort::new_composite(id.as_str()).map_err(|_| MarshalError::Cstring)?;
            for arg in args {
                let arg_sort = build_sort(arg)?;
                sort.add_argument(&arg_sort);
            }
            Ok(sort)
        }
        kore::Sort::Var(_) => Err(MarshalError::Unsupported("Sort::Var")),
    }
}

fn build_symbol(id: &kore::SymbolId, sorts: &[kore::Sort]) -> Result<Symbol, MarshalError> {
    let mut sym = Symbol::new(id.as_str()).map_err(|_| MarshalError::Cstring)?;
    for sort in sorts {
        let s = build_sort(sort)?;
        sym.add_formal_argument(&s);
    }
    Ok(sym)
}

fn variant_name(p: &kore::Pattern) -> &'static str {
    match p {
        kore::Pattern::Var(_) => "Var",
        kore::Pattern::SVar(_) => "SVar",
        kore::Pattern::Str(_) => "Str",
        kore::Pattern::App(_) => "App",
        kore::Pattern::LeftAssoc(_) => "LeftAssoc",
        kore::Pattern::RightAssoc(_) => "RightAssoc",
        kore::Pattern::Top(_) => "Top",
        kore::Pattern::Bottom(_) => "Bottom",
        kore::Pattern::Dv { .. } => "Dv",
        kore::Pattern::Not { .. } => "Not",
        kore::Pattern::Implies { .. } => "Implies",
        kore::Pattern::Iff { .. } => "Iff",
        kore::Pattern::And { .. } => "And",
        kore::Pattern::Or { .. } => "Or",
        kore::Pattern::Exists { .. } => "Exists",
        kore::Pattern::Forall { .. } => "Forall",
        kore::Pattern::Mu { .. } => "Mu",
        kore::Pattern::Nu { .. } => "Nu",
        kore::Pattern::Ceil { .. } => "Ceil",
        kore::Pattern::Floor { .. } => "Floor",
        kore::Pattern::Equals { .. } => "Equals",
        kore::Pattern::In { .. } => "In",
        kore::Pattern::Next { .. } => "Next",
        kore::Pattern::Rewrites { .. } => "Rewrites",
    }
}
