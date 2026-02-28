from kframework_py.kore.syntax import Sort, SortVar, SortApp

def test_sort():
    # Parsing
    sort = Sort.parse("SortA{SortB{SortC}, SortD, SortE{SortF, SortG{}}}")

    # Getters and Constructors don't re-allocate
    arg0 = sort.sorts[0]
    arg1 = sort.sorts[1]
    reused = SortApp("Reused", (arg0, arg1))
    assert arg0 is sort.sorts[0]
    assert arg1 is reused.sorts[1]

    # Dataclass repr
    expected_repr = "SortApp(name='SortB', sorts=(SortVar(name='SortC'),))"
    assert arg0.__repr__() == expected_repr

    # Structural equivalence
    app = SortApp("SortB", (SortVar("SortC"),))
    assert arg0 == app
