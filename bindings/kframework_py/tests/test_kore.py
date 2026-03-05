from kframework_py.kore.syntax import Sort, SortVar, SortApp
from kframework_py.kore.syntax import (
    Pattern, EVar, SVar, String, App, LeftAssoc, RightAssoc,
    Top, Bottom, DV, Not, Implies, Iff, And, Or,
    Exists, Forall, Mu, Nu, Ceil, Floor, Equals, In, Next, Rewrites,
)
from kframework_py.kore.syntax import (
    Sentence, Symbol, Import, SortDecl, SymbolDecl, AliasDecl, Axiom, Claim,
)
from kframework_py.kore.syntax import Module, Definition

def test_sort():
    # Parsing
    sort = Sort.parse("SortA{SortB{SortC}, SortD, SortE{SortF, SortG{}}}")

    # Getters and Constructors don't re-allocate
    arg0 = sort.sorts[0]
    arg1 = sort.sorts[1]
    reused = SortApp("Reused", (arg0, arg1))
    #assert arg0 is sort.sorts[0]
    #assert arg1 is reused.sorts[1]

    # Structural equivalence
    app = SortApp("SortB", (SortVar("SortC"),))
    assert arg0 == app

def test_sort_constructors():
    sort1 = SortVar("SortInt")
    sort2 = SortApp("SortInt", ())

def test_evar():
    sort = SortApp("SortInt")
    evar = EVar("x", sort)
    assert evar.name == "x"
    assert isinstance(evar, Pattern)
    assert isinstance(evar, EVar)
    assert "EVar" in repr(evar)

def test_svar():
    sort = SortApp("SortInt")
    svar = SVar("@x", sort)
    assert svar.name == "@x"
    assert isinstance(svar, Pattern)
    assert isinstance(svar, SVar)

def test_string():
    s = String("hello")
    assert s.value == "hello"
    assert isinstance(s, Pattern)

def test_app():
    sort = SortApp("SortInt")
    arg = EVar("x", sort)
    app = App("f", [sort], [arg])
    assert app.symbol == "f"
    assert len(app.sorts) == 1
    assert len(app.args) == 1
    assert isinstance(app, Pattern)

def test_app_defaults():
    app = App("f")
    assert app.symbol == "f"
    assert len(app.sorts) == 0
    assert len(app.args) == 0

def test_top_bottom():
    sort = SortApp("SortInt")
    top = Top(sort)
    bottom = Bottom(sort)
    assert isinstance(top, Pattern)
    assert isinstance(bottom, Pattern)
    assert "Top" in repr(top)
    assert "Bottom" in repr(bottom)

def test_not():
    sort = SortApp("SortInt")
    top = Top(sort)
    neg = Not(sort, top)
    assert isinstance(neg, Pattern)
    assert isinstance(neg, Not)
    assert isinstance(neg.pattern, Top)

def test_next():
    sort = SortApp("SortInt")
    top = Top(sort)
    nxt = Next(sort, top)
    assert isinstance(nxt, Pattern)
    assert isinstance(nxt.pattern, Top)

def test_implies():
    sort = SortApp("SortInt")
    top = Top(sort)
    bottom = Bottom(sort)
    imp = Implies(sort, top, bottom)
    assert isinstance(imp, Pattern)
    assert isinstance(imp.left, Top)
    assert isinstance(imp.right, Bottom)

def test_iff():
    sort = SortApp("SortInt")
    top = Top(sort)
    bottom = Bottom(sort)
    iff = Iff(sort, top, bottom)
    assert isinstance(iff, Pattern)

def test_rewrites():
    sort = SortApp("SortInt")
    top = Top(sort)
    bottom = Bottom(sort)
    rew = Rewrites(sort, top, bottom)
    assert isinstance(rew, Pattern)
    assert isinstance(rew.left, Top)
    assert isinstance(rew.right, Bottom)

def test_and_or():
    sort = SortApp("SortInt")
    top = Top(sort)
    bottom = Bottom(sort)
    conj = And(sort, [top, bottom])
    disj = Or(sort, [top, bottom])
    assert isinstance(conj, Pattern)
    assert isinstance(disj, Pattern)
    assert len(conj.ops) == 2
    assert len(disj.ops) == 2

def test_exists_forall():
    sort = SortApp("SortInt")
    evar = EVar("x", sort)
    top = Top(sort)
    ex = Exists(sort, evar, top)
    fa = Forall(sort, evar, top)
    assert isinstance(ex, Pattern)
    assert isinstance(fa, Pattern)
    assert isinstance(ex.var, EVar)
    assert isinstance(fa.var, EVar)

def test_mu_nu():
    sort = SortApp("SortInt")
    svar = SVar("@x", sort)
    top = Top(sort)
    mu = Mu(svar, top)
    nu = Nu(svar, top)
    assert isinstance(mu, Pattern)
    assert isinstance(nu, Pattern)
    assert isinstance(mu.var, SVar)
    assert isinstance(nu.var, SVar)

def test_ceil_floor():
    op_sort = SortApp("SortInt")
    sort = SortApp("SortBool")
    top = Top(op_sort)
    ceil = Ceil(op_sort, sort, top)
    floor = Floor(op_sort, sort, top)
    assert isinstance(ceil, Pattern)
    assert isinstance(floor, Pattern)
    assert isinstance(ceil.pattern, Top)

def test_equals_in():
    op_sort = SortApp("SortInt")
    sort = SortApp("SortBool")
    top = Top(op_sort)
    bottom = Bottom(op_sort)
    eq = Equals(op_sort, sort, top, bottom)
    in_ = In(op_sort, sort, top, bottom)
    assert isinstance(eq, Pattern)
    assert isinstance(in_, Pattern)
    assert isinstance(eq.left, Top)
    assert isinstance(in_.right, Bottom)

def test_dv():
    sort = SortApp("SortInt")
    value = "42"
    dv = DV(sort, value)
    assert isinstance(dv, Pattern)
    assert isinstance(dv.value, str)
    assert dv.value == "42"

def test_left_right_assoc():
    la = LeftAssoc("f", [], [])
    ra = RightAssoc("f", [], [])
    assert isinstance(la, Pattern)
    assert isinstance(ra, Pattern)

def test_parse_pattern():
    parsed = Pattern.parse("\\top{SortInt{}}()")
    assert isinstance(parsed, Top)

    parsed = Pattern.parse("x:SortInt{}")
    assert isinstance(parsed, EVar)
    assert parsed.name == "x"

def test_parse_app():
    parsed = Pattern.parse("f{}()")
    assert isinstance(parsed, App)
    assert parsed.symbol == "f"

def test_parse_not():
    parsed = Pattern.parse("\\not{SortInt{}}(\\top{SortInt{}}())")
    assert isinstance(parsed, Not)
    assert isinstance(parsed.pattern, Top)

# ==========================================
# Sentence tests
# ==========================================

def test_symbol():
    sym = Symbol("f")
    assert sym.name == "f"
    assert len(sym.vars) == 0
    assert "Symbol" in repr(sym)

def test_symbol_with_vars():
    sym = Symbol("f", [SortVar("S"), SortVar("T")])
    assert sym.name == "f"
    assert len(sym.vars) == 2
    assert isinstance(sym.vars[0], SortVar)

def test_import():
    imp = Import("MyModule")
    assert imp.module_name == "MyModule"
    assert len(imp.attrs) == 0
    assert isinstance(imp, Sentence)
    assert isinstance(imp, Import)
    assert "Import" in repr(imp)

def test_import_with_attrs():
    attr = App("attr")
    imp = Import("MyModule", [attr])
    assert len(imp.attrs) == 1
    assert isinstance(imp.attrs[0], App)

def test_sort_decl():
    sd = SortDecl("MySort", [SortVar("S")])
    assert sd.name == "MySort"
    assert len(sd.vars) == 1
    assert isinstance(sd.vars[0], SortVar)
    assert len(sd.attrs) == 0
    assert sd.hooked == False
    assert isinstance(sd, Sentence)

def test_sort_decl_hooked():
    sd = SortDecl("MySort", [], hooked=True)
    assert sd.hooked == True

def test_symbol_decl():
    sym = Symbol("f", [SortVar("S")])
    sort_int = SortApp("SortInt")
    sd = SymbolDecl(sym, [sort_int], sort_int)
    assert isinstance(sd, Sentence)
    assert isinstance(sd, SymbolDecl)
    assert sd.symbol.name == "f"
    assert len(sd.symbol.vars) == 1
    assert len(sd.param_sorts) == 1
    assert sd.hooked == False

def test_symbol_decl_hooked():
    sym = Symbol("f")
    sort_int = SortApp("SortInt")
    sd = SymbolDecl(sym, [], sort_int, hooked=True)
    assert sd.hooked == True

def test_alias_decl():
    alias_sym = Symbol("myAlias")
    sort_int = SortApp("SortInt")
    left = App("myAlias")
    right = Top(sort_int)
    ad = AliasDecl(alias_sym, [], sort_int, left, right)
    assert isinstance(ad, Sentence)
    assert isinstance(ad, AliasDecl)
    assert ad.alias.name == "myAlias"
    assert isinstance(ad.left, App)
    assert isinstance(ad.right, Top)

def test_axiom():
    sort_int = SortApp("SortInt")
    top = Top(sort_int)
    ax = Axiom([SortVar("S")], top)
    assert isinstance(ax, Sentence)
    assert isinstance(ax, Axiom)
    assert len(ax.vars) == 1
    assert isinstance(ax.vars[0], SortVar)
    assert isinstance(ax.pattern, Top)
    assert len(ax.attrs) == 0

def test_axiom_with_attrs():
    sort_int = SortApp("SortInt")
    top = Top(sort_int)
    attr = App("myAttr")
    ax = Axiom([], top, [attr])
    assert len(ax.attrs) == 1
    assert isinstance(ax.attrs[0], App)

def test_claim():
    sort_int = SortApp("SortInt")
    top = Top(sort_int)
    cl = Claim([SortVar("S")], top)
    assert isinstance(cl, Sentence)
    assert isinstance(cl, Claim)
    assert len(cl.vars) == 1
    assert isinstance(cl.pattern, Top)

def test_parse_import():
    parsed = Sentence.parse("import MyModule []")
    assert isinstance(parsed, Import)
    assert parsed.module_name == "MyModule"

def test_parse_sort_decl():
    parsed = Sentence.parse("sort MySort{S} []")
    assert isinstance(parsed, SortDecl)
    assert parsed.name == "MySort"
    assert len(parsed.vars) == 1

def test_parse_axiom():
    parsed = Sentence.parse("axiom{S} \\top{SortInt{}}() []")
    assert isinstance(parsed, Axiom)
    assert len(parsed.vars) == 1
    assert isinstance(parsed.pattern, Top)

# ==========================================
# Module tests
# ==========================================

def test_module():
    mod = Module("MyModule")
    assert mod.name == "MyModule"
    assert len(mod.sentences) == 0
    assert len(mod.attrs) == 0
    assert "Module" in repr(mod)

def test_module_with_sentences():
    sort_int = SortApp("SortInt")
    top = Top(sort_int)
    ax = Axiom([], top)
    imp = Import("OtherModule")
    mod = Module("MyModule", [imp, ax])
    assert mod.name == "MyModule"
    assert len(mod.sentences) == 2
    assert isinstance(mod.sentences[0], Import)
    assert isinstance(mod.sentences[1], Axiom)

def test_module_with_attrs():
    attr = App("myAttr")
    mod = Module("MyModule", [], [attr])
    assert len(mod.attrs) == 1
    assert isinstance(mod.attrs[0], App)

def test_module_parse():
    parsed = Module.parse("module MyModule endmodule []")
    assert parsed.name == "MyModule"
    assert len(parsed.sentences) == 0

def test_module_parse_with_sentences():
    text = "module MyModule import Other [] sort MySort{} [] endmodule []"
    parsed = Module.parse(text)
    assert parsed.name == "MyModule"
    assert len(parsed.sentences) == 2
    assert isinstance(parsed.sentences[0], Import)
    assert isinstance(parsed.sentences[1], SortDecl)

# ==========================================
# Definition tests
# ==========================================

def test_definition():
    defn = Definition()
    assert len(defn.modules) == 0
    assert len(defn.attrs) == 0
    assert "Definition" in repr(defn)

def test_definition_with_modules():
    mod1 = Module("ModA")
    mod2 = Module("ModB")
    defn = Definition([mod1, mod2])
    assert len(defn.modules) == 2
    assert defn.modules[0].name == "ModA"
    assert defn.modules[1].name == "ModB"

def test_definition_with_attrs():
    attr = App("myAttr")
    defn = Definition([], [attr])
    assert len(defn.attrs) == 1
    assert isinstance(defn.attrs[0], App)

def test_definition_parse():
    text = "[] module MyModule endmodule []"
    parsed = Definition.parse(text)
    assert len(parsed.modules) == 1
    assert parsed.modules[0].name == "MyModule"

def test_definition_nested():
    sort_int = SortApp("SortInt")
    top = Top(sort_int)
    ax = Axiom([SortVar("S")], top, [App("label")])
    mod = Module("MyModule", [ax], [App("modAttr")])
    defn = Definition([mod], [App("defAttr")])
    assert len(defn.modules) == 1
    assert len(defn.modules[0].sentences) == 1
    assert isinstance(defn.modules[0].sentences[0], Axiom)
    assert len(defn.modules[0].attrs) == 1
    assert len(defn.attrs) == 1
