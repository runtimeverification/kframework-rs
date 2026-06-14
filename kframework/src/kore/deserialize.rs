use super::{App, Id, Pattern, SVar, SetVarId, Sort, Str, SymbolId, Var};
use serde;
use serde::de::{Deserialize, Deserializer, Error, MapAccess, Unexpected, Visitor};
use std::fmt;
use std::sync::Arc;

macro_rules! deserialize_for_id {
    ($struct:ident, $expecting:expr, $valid_value:expr) => {
        impl<'de> Deserialize<'de> for $struct {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct VisitorImpl;

                impl<'de> Visitor<'de> for VisitorImpl {
                    type Value = $struct;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str($expecting)
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: Error,
                    {
                        $struct::new(v.into())
                            .map_err(|_e| Error::invalid_value(Unexpected::Str(v), &$valid_value))
                    }
                }

                deserializer.deserialize_str(VisitorImpl)
            }
        }
    };
}

deserialize_for_id!(Id, "struct Id", "an identifier");
deserialize_for_id!(SymbolId, "struct SymbolId", "a symbol identifier");

impl<'de> Deserialize<'de> for Sort {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Field {
            Tag,
            Name,
            Args,
        }

        #[derive(serde::Deserialize)]
        #[serde(variant_identifier)]
        enum Tag {
            SortVar,
            SortApp,
        }

        struct SortVisitor;

        impl<'de> Visitor<'de> for SortVisitor {
            type Value = Sort;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum Sort")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut tag: Option<Tag> = None;
                let mut id: Option<Id> = None;
                let mut args: Option<Vec<Sort>> = None;
                while let Some(field) = map.next_key()? {
                    match field {
                        Field::Tag => {
                            if tag.is_some() {
                                return Err(Error::duplicate_field("tag"));
                            }
                            tag = Some(map.next_value()?);
                        }
                        Field::Name => {
                            if id.is_some() {
                                return Err(Error::duplicate_field("name"));
                            }
                            id = Some(map.next_value()?);
                        }
                        Field::Args => {
                            if args.is_some() {
                                return Err(Error::duplicate_field("args"));
                            }
                            args = Some(map.next_value()?);
                        }
                    }
                }
                let tag = tag.ok_or_else(|| Error::missing_field("tag"))?;
                let id = id.ok_or_else(|| Error::missing_field("name"))?;
                match tag {
                    Tag::SortVar => Ok(Sort::Var(id)),
                    Tag::SortApp => {
                        let args = args.ok_or_else(|| Error::missing_field("args"))?;
                        let args = args.into_iter().map(Arc::new).collect::<Box<_>>();
                        Ok(Sort::App { id, args })
                    }
                }
            }
        }

        const FIELDS: &[&str] = &["tag", "name", "args"];
        deserializer.deserialize_struct("Sort", FIELDS, SortVisitor)
    }
}

impl<'de> Deserialize<'de> for Pattern {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(serde::Deserialize, PartialEq, Eq)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Field {
            Tag,
            Symbol,
            Name,
            Var,
            Value,
            Sort,
            ArgSort,
            VarSort,
            Sorts,
            Arg,
            First,
            Second,
            Dest,
            Source,
            Args,
            Argss,
            Patterns,
        }

        #[derive(serde::Deserialize)]
        #[serde(variant_identifier)]
        enum Tag {
            EVar,
            SVar,
            String,
            App,
            LeftAssoc,
            RightAssoc,
            Top,
            Bottom,
            DV,
            Not,
            Implies,
            Iff,
            And,
            Or,
            Exists,
            Forall,
            Mu,
            Nu,
            Ceil,
            Floor,
            Equals,
            In,
            Next,
            Rewrites,
        }

        struct PatternVisitor;

        impl<'de> Visitor<'de> for PatternVisitor {
            type Value = Pattern;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum Pattern")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut tag: Option<Tag> = None;

                let mut symbol: Option<SymbolId> = None;

                let mut name: Option<String> = None; // can be Id or SymbolId depending on context
                let mut var: Option<String> = None; // can be Id or SetVarId depending on context
                let mut value: Option<String> = None;

                let mut sort: Option<Sort> = None;
                let mut arg_sort: Option<Sort> = None;
                let mut var_sort: Option<Sort> = None;
                let mut sorts: Option<Vec<Sort>> = None;

                let mut arg: Option<Pattern> = None;
                let mut first: Option<Pattern> = None;
                let mut second: Option<Pattern> = None;
                let mut source: Option<Pattern> = None;
                let mut dest: Option<Pattern> = None;

                let mut args: Option<Vec<Pattern>> = None;
                let mut argss: Option<Vec<Pattern>> = None;
                let mut patterns: Option<Vec<Pattern>> = None;

                macro_rules! set_field {
                    ($field:expr , { $($case:pat => $var:ident, $field_name:expr,)*}) => {
                        match $field {
                        $(
                            $case => {
                                if $var.is_some() {
                                    return Err(Error::duplicate_field($field_name));
                                }
                                $var = Some(map.next_value()?);
                            }
                        )*
                        }
                    }
                }

                while let Some(field) = map.next_key()? {
                    set_field!(field, {
                        Field::Tag => tag, "tag",
                        Field::Symbol => symbol, "symbol",
                        Field::Name => name, "name",
                        Field::Var => var, "var",
                        Field::Value => value, "value",
                        Field::Sort => sort, "sort",
                        Field::ArgSort => arg_sort, "argSort",
                        Field::VarSort => var_sort, "varSort",
                        Field::Sorts => sorts, "sorts",
                        Field::Arg => arg, "arg",
                        Field::First => first, "first",
                        Field::Second => second, "second",
                        Field::Dest => dest, "dest",
                        Field::Source => source, "source",
                        Field::Args => args, "args",
                        Field::Argss => argss, "argss",
                        Field::Patterns => patterns, "patterns",
                    })
                }

                macro_rules! field_or_missing {
                    ($field:ident, $name:expr) => {
                        $field.ok_or_else(|| Error::missing_field($name))?
                    };
                }

                macro_rules! id_or_invalid {
                    // TODO IdError { text, offset } to avoid clone
                    ($struct:ident, $id:expr, $valid_value:expr) => {
                        $struct::new($id.clone()).map_err(|_e| {
                            Error::invalid_value(Unexpected::Str(&$id), &$valid_value)
                        })?
                    };
                }

                let tag = field_or_missing!(tag, "tag");
                match tag {
                    Tag::EVar => {
                        let name = field_or_missing!(name, "name");
                        let sort = field_or_missing!(sort, "sort");
                        let name = id_or_invalid!(Id, name, "an identifier");
                        Ok(Pattern::Var(Var { id: name, sort }))
                    }
                    Tag::SVar => {
                        let name = field_or_missing!(name, "name");
                        let sort = field_or_missing!(sort, "sort");
                        let name = id_or_invalid!(SetVarId, name, "a set variable identifier");
                        Ok(Pattern::SVar(SVar { id: name, sort }))
                    }
                    Tag::String => {
                        let value = field_or_missing!(value, "value");
                        Ok(Pattern::Str(Str(value)))
                    }
                    Tag::App => {
                        let name = field_or_missing!(name, "name");
                        let sorts = field_or_missing!(sorts, "sorts");
                        let args = field_or_missing!(args, "args");
                        let name = id_or_invalid!(SymbolId, name, "a symbol identifier");
                        Ok(Pattern::App(App {
                            symbol: name,
                            sorts,
                            args,
                        }))
                    }
                    Tag::LeftAssoc => {
                        let symbol = field_or_missing!(symbol, "symbol");
                        let sorts = field_or_missing!(sorts, "sorts");
                        let argss = field_or_missing!(argss, "argss");
                        Ok(Pattern::LeftAssoc(App {
                            symbol,
                            sorts,
                            args: argss,
                        }))
                    }
                    Tag::RightAssoc => {
                        let symbol = field_or_missing!(symbol, "symbol");
                        let sorts = field_or_missing!(sorts, "sorts");
                        let argss = field_or_missing!(argss, "argss");
                        Ok(Pattern::RightAssoc(App {
                            symbol,
                            sorts,
                            args: argss,
                        }))
                    }
                    Tag::Top => {
                        let sort = field_or_missing!(sort, "sort");
                        Ok(Pattern::Top(sort))
                    }
                    Tag::Bottom => {
                        let sort = field_or_missing!(sort, "sort");
                        Ok(Pattern::Bottom(sort))
                    }
                    Tag::DV => {
                        let sort = field_or_missing!(sort, "sort");
                        let value = field_or_missing!(value, "value");
                        Ok(Pattern::Dv {
                            sort,
                            value: Str(value),
                        })
                    }
                    Tag::Not => {
                        let sort = field_or_missing!(sort, "sort");
                        let arg = field_or_missing!(arg, "arg");
                        Ok(Pattern::Not {
                            sort,
                            op: Box::new(arg),
                        })
                    }
                    Tag::Implies => {
                        let sort = field_or_missing!(sort, "sort");
                        let first = field_or_missing!(first, "first");
                        let second = field_or_missing!(second, "second");
                        Ok(Pattern::Implies {
                            sort,
                            left: Box::new(first),
                            right: Box::new(second),
                        })
                    }
                    Tag::Iff => {
                        let sort = field_or_missing!(sort, "sort");
                        let first = field_or_missing!(first, "first");
                        let second = field_or_missing!(second, "second");
                        Ok(Pattern::Iff {
                            sort,
                            left: Box::new(first),
                            right: Box::new(second),
                        })
                    }
                    Tag::And => {
                        let sort = field_or_missing!(sort, "sort");
                        let patterns = field_or_missing!(patterns, "patterns");
                        Ok(Pattern::And {
                            sort,
                            ops: patterns,
                        })
                    }
                    Tag::Or => {
                        let sort = field_or_missing!(sort, "sort");
                        let patterns = field_or_missing!(patterns, "patterns");
                        Ok(Pattern::Or {
                            sort,
                            ops: patterns,
                        })
                    }
                    Tag::Exists => {
                        let sort = field_or_missing!(sort, "sort");
                        let var = field_or_missing!(var, "var");
                        let var_sort = field_or_missing!(var_sort, "varSort");
                        let arg = field_or_missing!(arg, "arg");
                        let var = id_or_invalid!(Id, var, "an identifier");
                        Ok(Pattern::Exists {
                            sort,
                            var: Var {
                                id: var,
                                sort: var_sort,
                            },
                            op: Box::new(arg),
                        })
                    }
                    Tag::Forall => {
                        let sort = field_or_missing!(sort, "sort");
                        let var = field_or_missing!(var, "var");
                        let var_sort = field_or_missing!(var_sort, "varSort");
                        let arg = field_or_missing!(arg, "arg");
                        let var = id_or_invalid!(Id, var, "an identifier");
                        Ok(Pattern::Forall {
                            sort,
                            var: Var {
                                id: var,
                                sort: var_sort,
                            },
                            op: Box::new(arg),
                        })
                    }
                    Tag::Mu => {
                        let var = field_or_missing!(var, "var");
                        let var_sort = field_or_missing!(var_sort, "varSort");
                        let arg = field_or_missing!(arg, "arg");
                        let var = id_or_invalid!(SetVarId, var, "a set variable identifier");
                        Ok(Pattern::Mu {
                            var: SVar {
                                id: var,
                                sort: var_sort,
                            },
                            op: Box::new(arg),
                        })
                    }
                    Tag::Nu => {
                        let var = field_or_missing!(var, "var");
                        let var_sort = field_or_missing!(var_sort, "varSort");
                        let arg = field_or_missing!(arg, "arg");
                        let var = id_or_invalid!(SetVarId, var, "a set variable identifier");
                        Ok(Pattern::Nu {
                            var: SVar {
                                id: var,
                                sort: var_sort,
                            },
                            op: Box::new(arg),
                        })
                    }
                    Tag::Ceil => {
                        let arg_sort = field_or_missing!(arg_sort, "argSort");
                        let sort = field_or_missing!(sort, "sort");
                        let arg = field_or_missing!(arg, "arg");
                        Ok(Pattern::Ceil {
                            op_sort: arg_sort,
                            sort,
                            op: Box::new(arg),
                        })
                    }
                    Tag::Floor => {
                        let arg_sort = field_or_missing!(arg_sort, "argSort");
                        let sort = field_or_missing!(sort, "sort");
                        let arg = field_or_missing!(arg, "arg");
                        Ok(Pattern::Floor {
                            op_sort: arg_sort,
                            sort,
                            op: Box::new(arg),
                        })
                    }
                    Tag::Equals => {
                        let arg_sort = field_or_missing!(arg_sort, "argSort");
                        let sort = field_or_missing!(sort, "sort");
                        let first = field_or_missing!(first, "first");
                        let second = field_or_missing!(second, "second");
                        Ok(Pattern::Equals {
                            op_sort: arg_sort,
                            sort,
                            left: Box::new(first),
                            right: Box::new(second),
                        })
                    }
                    Tag::In => {
                        let arg_sort = field_or_missing!(arg_sort, "argSort");
                        let sort = field_or_missing!(sort, "sort");
                        let first = field_or_missing!(first, "first");
                        let second = field_or_missing!(second, "second");
                        Ok(Pattern::In {
                            op_sort: arg_sort,
                            sort,
                            left: Box::new(first),
                            right: Box::new(second),
                        })
                    }
                    Tag::Next => {
                        let sort = field_or_missing!(sort, "sort");
                        let dest = field_or_missing!(dest, "dest");
                        Ok(Pattern::Next {
                            sort,
                            op: Box::new(dest),
                        })
                    }
                    Tag::Rewrites => {
                        let sort = field_or_missing!(sort, "sort");
                        let source = field_or_missing!(source, "source");
                        let dest = field_or_missing!(dest, "dest");
                        Ok(Pattern::Rewrites {
                            sort,
                            left: Box::new(source),
                            right: Box::new(dest),
                        })
                    }
                }
            }
        }

        const FIELDS: &[&str] = &[
            "tag", "name", "symbol", "var", "value", "sort", "argSort", "varSort", "sorts", "arg",
            "first", "second", "dest", "source", "args", "argss", "patterns",
        ];
        deserializer.deserialize_struct("Pattern", FIELDS, PatternVisitor)
    }
}
