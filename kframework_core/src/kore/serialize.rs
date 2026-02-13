use super::visitor::PatternVisitor;
use super::{App, Pattern, SVar, Sort, Str, Var};
use serde::ser::{Serialize, SerializeStruct, Serializer};

impl Serialize for Sort {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Sort::Var(id) => {
                let mut state = serializer.serialize_struct("SortVar", 2)?;
                state.serialize_field("tag", "SortVar")?;
                state.serialize_field("name", &id.0)?;
                state.end()
            }
            Sort::App { id, args } => {
                let mut state = serializer.serialize_struct("SortApp", 3)?;
                state.serialize_field("tag", "SortApp")?;
                state.serialize_field("name", &id.0)?;
                state.serialize_field("args", args)?;
                state.end()
            }
        }
    }
}

impl Serialize for Pattern {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        struct Visitor<S>(S);

        impl<S: Serializer> PatternVisitor<Result<S::Ok, S::Error>> for Visitor<S> {
            fn visit_var(self, var: &Var) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("EVar", 3)?;
                state.serialize_field("tag", "EVar")?;
                state.serialize_field("name", &var.id.0)?;
                state.serialize_field("sort", &var.sort)?;
                state.end()
            }

            fn visit_svar(self, svar: &SVar) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("SVar", 3)?;
                state.serialize_field("tag", "SVar")?;
                state.serialize_field("name", &svar.id.0)?;
                state.serialize_field("sort", &svar.sort)?;
                state.end()
            }

            fn visit_str(self, s: &Str) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("String", 2)?;
                state.serialize_field("tag", "String")?;
                state.serialize_field("value", &s.0)?;
                state.end()
            }

            fn visit_app(self, app: &App) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("App", 4)?;
                state.serialize_field("tag", "App")?;
                state.serialize_field("name", &app.symbol.0)?;
                state.serialize_field("sorts", &app.sorts)?;
                state.serialize_field("args", &app.args)?;
                state.end()
            }

            fn visit_left_assoc(self, app: &App) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("LeftAssoc", 4)?;
                state.serialize_field("tag", "LeftAssoc")?;
                state.serialize_field("symbol", &app.symbol.0)?;
                state.serialize_field("sorts", &app.sorts)?;
                state.serialize_field("argss", &app.args)?;
                state.end()
            }

            fn visit_right_assoc(self, app: &App) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("RightAssoc", 4)?;
                state.serialize_field("tag", "RightAssoc")?;
                state.serialize_field("symbol", &app.symbol.0)?;
                state.serialize_field("sorts", &app.sorts)?;
                state.serialize_field("argss", &app.args)?;
                state.end()
            }

            fn visit_top(self, sort: &Sort) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("Top", 2)?;
                state.serialize_field("tag", "Top")?;
                state.serialize_field("sort", sort)?;
                state.end()
            }

            fn visit_bottom(self, sort: &Sort) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("Bottom", 2)?;
                state.serialize_field("tag", "Bottom")?;
                state.serialize_field("sort", sort)?;
                state.end()
            }

            fn visit_dv(self, sort: &Sort, value: &Str) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("DV", 3)?;
                state.serialize_field("tag", "DV")?;
                state.serialize_field("sort", sort)?;
                state.serialize_field("value", &value.0)?;
                state.end()
            }

            fn visit_not(self, sort: &Sort, op: &Pattern) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("Not", 3)?;
                state.serialize_field("tag", "Not")?;
                state.serialize_field("sort", sort)?;
                state.serialize_field("arg", op)?;
                state.end()
            }

            fn visit_implies(
                self,
                sort: &Sort,
                left: &Pattern,
                right: &Pattern,
            ) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("Implies", 4)?;
                state.serialize_field("tag", "Implies")?;
                state.serialize_field("sort", sort)?;
                state.serialize_field("first", left)?;
                state.serialize_field("second", right)?;
                state.end()
            }

            fn visit_iff(
                self,
                sort: &Sort,
                left: &Pattern,
                right: &Pattern,
            ) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("Iff", 4)?;
                state.serialize_field("tag", "Iff")?;
                state.serialize_field("sort", sort)?;
                state.serialize_field("first", left)?;
                state.serialize_field("second", right)?;
                state.end()
            }

            fn visit_and(self, sort: &Sort, ops: &[Pattern]) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("And", 3)?;
                state.serialize_field("tag", "And")?;
                state.serialize_field("sort", sort)?;
                state.serialize_field("patterns", ops)?;
                state.end()
            }

            fn visit_or(self, sort: &Sort, ops: &[Pattern]) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("Or", 3)?;
                state.serialize_field("tag", "Or")?;
                state.serialize_field("sort", sort)?;
                state.serialize_field("patterns", ops)?;
                state.end()
            }

            fn visit_exists(self, sort: &Sort, var: &Var, op: &Pattern) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("Exists", 5)?;
                state.serialize_field("tag", "Exists")?;
                state.serialize_field("sort", sort)?;
                state.serialize_field("var", &var.id.0)?;
                state.serialize_field("varSort", &var.sort)?;
                state.serialize_field("arg", op)?;
                state.end()
            }

            fn visit_forall(self, sort: &Sort, var: &Var, op: &Pattern) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("Forall", 5)?;
                state.serialize_field("tag", "Forall")?;
                state.serialize_field("sort", sort)?;
                state.serialize_field("var", &var.id.0)?;
                state.serialize_field("varSort", &var.sort)?;
                state.serialize_field("arg", op)?;
                state.end()
            }

            fn visit_mu(self, var: &SVar, op: &Pattern) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("Mu", 4)?;
                state.serialize_field("tag", "Mu")?;
                state.serialize_field("var", &var.id.0)?;
                state.serialize_field("varSort", &var.sort)?;
                state.serialize_field("arg", op)?;
                state.end()
            }

            fn visit_nu(self, var: &SVar, op: &Pattern) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("Nu", 4)?;
                state.serialize_field("tag", "Nu")?;
                state.serialize_field("var", &var.id.0)?;
                state.serialize_field("varSort", &var.sort)?;
                state.serialize_field("arg", op)?;
                state.end()
            }

            fn visit_ceil(
                self,
                op_sort: &Sort,
                sort: &Sort,
                op: &Pattern,
            ) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("Ceil", 4)?;
                state.serialize_field("tag", "Ceil")?;
                state.serialize_field("argSort", op_sort)?;
                state.serialize_field("sort", sort)?;
                state.serialize_field("arg", op)?;
                state.end()
            }

            fn visit_floor(
                self,
                op_sort: &Sort,
                sort: &Sort,
                op: &Pattern,
            ) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("Floor", 4)?;
                state.serialize_field("tag", "Floor")?;
                state.serialize_field("argSort", op_sort)?;
                state.serialize_field("sort", sort)?;
                state.serialize_field("arg", op)?;
                state.end()
            }

            fn visit_equals(
                self,
                op_sort: &Sort,
                sort: &Sort,
                left: &Pattern,
                right: &Pattern,
            ) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("Equals", 5)?;
                state.serialize_field("tag", "Equals")?;
                state.serialize_field("argSort", op_sort)?;
                state.serialize_field("sort", sort)?;
                state.serialize_field("first", left)?;
                state.serialize_field("second", right)?;
                state.end()
            }

            fn visit_in(
                self,
                op_sort: &Sort,
                sort: &Sort,
                left: &Pattern,
                right: &Pattern,
            ) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("In", 5)?;
                state.serialize_field("tag", "In")?;
                state.serialize_field("argSort", op_sort)?;
                state.serialize_field("sort", sort)?;
                state.serialize_field("first", left)?;
                state.serialize_field("second", right)?;
                state.end()
            }

            fn visit_next(self, sort: &Sort, op: &Pattern) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("Next", 3)?;
                state.serialize_field("tag", "Next")?;
                state.serialize_field("sort", sort)?;
                state.serialize_field("dest", op)?;
                state.end()
            }

            fn visit_rewrites(
                self,
                sort: &Sort,
                left: &Pattern,
                right: &Pattern,
            ) -> Result<S::Ok, S::Error> {
                let mut state = self.0.serialize_struct("Rewrites", 4)?;
                state.serialize_field("tag", "Rewrites")?;
                state.serialize_field("sort", sort)?;
                state.serialize_field("source", left)?;
                state.serialize_field("dest", right)?;
                state.end()
            }
        }

        self.accept(Visitor(serializer))
    }
}
