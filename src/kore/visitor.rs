use super::syntax::{App, Pattern, SVar, Sort, Str, Var};

pub trait PatternVisitor<T> {
    fn visit_var(self, var: &Var) -> T;
    fn visit_svar(self, svar: &SVar) -> T;
    fn visit_str(self, s: &Str) -> T;
    fn visit_app(self, app: &App) -> T;
    fn visit_left_assoc(self, app: &App) -> T;
    fn visit_right_assoc(self, app: &App) -> T;
    fn visit_top(self, sort: &Sort) -> T;
    fn visit_bottom(self, sort: &Sort) -> T;
    fn visit_dv(self, sort: &Sort, value: &Str) -> T;
    fn visit_not(self, sort: &Sort, op: &Pattern) -> T;
    fn visit_implies(self, sort: &Sort, left: &Pattern, right: &Pattern) -> T;
    fn visit_iff(self, sort: &Sort, left: &Pattern, right: &Pattern) -> T;
    fn visit_and(self, sort: &Sort, ops: &[Pattern]) -> T;
    fn visit_or(self, sort: &Sort, ops: &[Pattern]) -> T;
    fn visit_exists(self, sort: &Sort, var: &Var, op: &Pattern) -> T;
    fn visit_forall(self, sort: &Sort, var: &Var, op: &Pattern) -> T;
    fn visit_mu(self, var: &SVar, op: &Pattern) -> T;
    fn visit_nu(self, var: &SVar, op: &Pattern) -> T;
    fn visit_ceil(self, op_sort: &Sort, sort: &Sort, op: &Pattern) -> T;
    fn visit_floor(self, op_sort: &Sort, sort: &Sort, op: &Pattern) -> T;
    fn visit_equals(self, op_sort: &Sort, sort: &Sort, left: &Pattern, right: &Pattern) -> T;
    fn visit_in(self, op_sort: &Sort, sort: &Sort, left: &Pattern, right: &Pattern) -> T;
    fn visit_next(self, sort: &Sort, op: &Pattern) -> T;
    fn visit_rewrites(self, sort: &Sort, left: &Pattern, right: &Pattern) -> T;
}

impl Pattern {
    pub fn accept<T>(&self, visitor: impl PatternVisitor<T>) -> T {
        match self {
            Pattern::Var(var) => visitor.visit_var(var),
            Pattern::SVar(var) => visitor.visit_svar(var),
            Pattern::Str(s) => visitor.visit_str(s),
            Pattern::App(app) => visitor.visit_app(app),
            Pattern::LeftAssoc(app) => visitor.visit_left_assoc(app),
            Pattern::RightAssoc(app) => visitor.visit_right_assoc(app),
            Pattern::Top(sort) => visitor.visit_top(sort),
            Pattern::Bottom(sort) => visitor.visit_bottom(sort),
            Pattern::Dv { sort, value } => visitor.visit_dv(sort, value),
            Pattern::Not { sort, op } => visitor.visit_not(sort, op),
            Pattern::Implies { sort, left, right } => visitor.visit_implies(sort, left, right),
            Pattern::Iff { sort, left, right } => visitor.visit_iff(sort, left, right),
            Pattern::And { sort, ops } => visitor.visit_and(sort, ops),
            Pattern::Or { sort, ops } => visitor.visit_or(sort, ops),
            Pattern::Exists { sort, var, op } => visitor.visit_exists(sort, var, op),
            Pattern::Forall { sort, var, op } => visitor.visit_forall(sort, var, op),
            Pattern::Mu { var, op } => visitor.visit_mu(var, op),
            Pattern::Nu { var, op } => visitor.visit_nu(var, op),
            Pattern::Ceil { op_sort, sort, op } => visitor.visit_ceil(op_sort, sort, op),
            Pattern::Floor { op_sort, sort, op } => visitor.visit_floor(op_sort, sort, op),
            Pattern::Equals {
                op_sort,
                sort,
                left,
                right,
            } => visitor.visit_equals(op_sort, sort, left, right),
            Pattern::In {
                op_sort,
                sort,
                left,
                right,
            } => visitor.visit_in(op_sort, sort, left, right),
            Pattern::Next { sort, op } => visitor.visit_next(sort, op),
            Pattern::Rewrites { sort, left, right } => visitor.visit_rewrites(sort, left, right),
        }
    }
}
