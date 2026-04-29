use super::ffi;
use super::{Pattern, Sort, Symbol};
use kframework::kore;
use std::collections::HashMap;

#[derive(Debug)]
pub enum MarshalError {
    Unsupported(&'static str),
    UnknownVar(String),
    Cstring,
}

pub trait VarHandler {
    fn substitute(&mut self, name: &str) -> Result<kore::Pattern, MarshalError>;
}

pub struct Marshaller<H: VarHandler> {
    /// SymbolId string -> Symbol. Each Symbol's underlying C++ AST is held
    /// alive via `shared_ptr` by every pattern that referenced it during
    /// construction; dropping the Symbol wrapper here only frees the C
    /// struct.
    symbols: HashMap<String, Symbol>,
    /// `*const kore::Pattern` (pointer identity into the caller-owned
    /// source template) -> Pattern. Only var-free subtrees of the source
    /// template are cached; substituted-Var subtrees live on the stack
    /// and must not be cached.
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

    pub fn set_handler(&mut self, h: H) {
        self.handler = Some(h);
    }

    pub fn marshal(&mut self, root: &kore::Pattern) -> Result<Pattern, MarshalError> {
        let (raw, _var_free, fresh) = self.marshal_node(root, false)?;
        // If the root resolved to a cached pointer (rare for fuzzer
        // templates, since they have Vars), transfer ownership out of the
        // cache so Pattern's Drop is the sole freer.
        if !fresh {
            if let Some(owned) = self.subtrees.remove(&(root as *const kore::Pattern)) {
                std::mem::forget(owned);
            }
        }
        Ok(Pattern::from_raw(raw))
    }

    /// Returns `(ptr, var_free, fresh)`.
    /// - `fresh = true`  => caller owns the C wrapper and must free it
    ///   (typically right after handing it to `add_argument`, since the
    ///   parent already took a `shared_ptr` copy of the underlying AST).
    /// - `fresh = false` => `ptr` is borrowed (lives in `self.subtrees`).
    fn marshal_node(
        &mut self,
        p: &kore::Pattern,
        was_var: bool,
    ) -> Result<(*mut ffi::kore_pattern, bool, bool), MarshalError> {
        if let Some(cached) = self.subtrees.get(&(p as *const kore::Pattern)) {
            return Ok((cached.pattern as *mut _, true, false));
        }

        let res = match p {
            kore::Pattern::Var(v) => {
                let handler = self
                    .handler
                    .as_mut()
                    .ok_or_else(|| MarshalError::UnknownVar(v.id.as_str().into()))?;
                let sub = handler.substitute(v.id.as_str())?;
                let (ptr, _, fresh) = self.marshal_node(&sub, true)?;
                Ok((ptr, false, fresh))
            }

            kore::Pattern::Dv { sort, value } => {
                let s = build_sort(sort)?;
                let pat =
                    Pattern::new_token(value.0.as_str(), &s).map_err(|_| MarshalError::Cstring)?;
                let raw = pat.pattern as *mut _;
                // Keep tracking the raw pointer manually until the
                // post-match step decides whether to cache or hand back.
                std::mem::forget(pat);
                Ok((raw, true, true))
            }

            kore::Pattern::App(app) => self.marshal_app(app),

            other => Err(MarshalError::Unsupported(variant_name(other))),
        };

        let (ptr, var_free, mut fresh) = res?;
        if !was_var && var_free && fresh {
            self.subtrees
                .insert(p as *const kore::Pattern, Pattern::from_raw(ptr));
            fresh = false;
        }
        Ok((ptr, var_free, fresh))
    }

    fn marshal_app(
        &mut self,
        app: &kore::App,
    ) -> Result<(*mut ffi::kore_pattern, bool, bool), MarshalError> {
        let sym_ptr = self.intern_symbol(&app.symbol, &app.sorts)?;
        let pat = unsafe { ffi::kore_composite_pattern_from_symbol(sym_ptr) };

        let mut var_free = true;
        for arg in &app.args {
            let (child, child_vf, child_fresh) = self.marshal_node(arg, false)?;
            unsafe { ffi::kore_composite_pattern_add_argument(pat, child) };
            // add_argument copied the inner shared_ptr; the underlying AST
            // is now retained by `pat`. A fresh child wrapper is no longer
            // needed and must be freed here, otherwise every var-bearing
            // intermediate node leaks its C wrapper struct each iteration.
            // Borrowed (cached) children must NOT be freed.
            if child_fresh {
                unsafe { ffi::kore_pattern_free(child) };
            }
            var_free &= child_vf;
        }

        Ok((pat, var_free, true))
    }

    fn intern_symbol(
        &mut self,
        id: &kore::SymbolId,
        sorts: &[kore::Sort],
    ) -> Result<*mut ffi::kore_symbol, MarshalError> {
        let key = id.as_str().to_owned();
        if let Some(sym) = self.symbols.get(&key) {
            return Ok(sym.symbol);
        }
        let mut sym = Symbol::new(id.as_str()).map_err(|_| MarshalError::Cstring)?;
        for sort in sorts {
            let s = build_sort(sort)?;
            sym.add_formal_argument(&s);
        }
        let raw = sym.symbol;
        self.symbols.insert(key, sym);
        Ok(raw)
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
