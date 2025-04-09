requires "imp.k"

module FUZZ
  imports IMP

  configuration <T/>

  syntax Stmt ::= "Assert" "(" BExp ")" ";" [strict]
  rule Assert( true ); => .K

  syntax Pgm   ::= "init_fuzz" "(" Int "," Int ")" [symbol(init_fuzz)]
  syntax TCell ::= "FUZZ_SUCCESS" [symbol(fuzz_success)]
                 | "FUZZ_FAILURE" [symbol(fuzz_failure)]

  syntax Id ::= "x" [token]
              | "y" [token]
              | "combined" [token]
              | "salted" [token]
              | "z" [token]
              | "hash_val" [token]

  rule <k> init_fuzz(X, Y)
        => int x, y, combined, salted, z, hash_val;

           x = X;
           y = Y;

           combined = x * 1000000 + y;

           salted = combined + 1;

           z = salted + 11400714819323198485;
           z = (z ^ (z >> 30)) * 13787848793156543929;
           z = (z ^ (z >> 27)) * 10723151780598845931;
           z = z ^ (z >> 31);

           hash_val = z & 4294967295;

           Assert( !( hash_val <= 15000 ) );
           ...
       </k>

  rule <T> <k> .K </k> ... </T> => FUZZ_SUCCESS
  rule <T> <k> Assert(false); ... </k> ... </T> => FUZZ_FAILURE
endmodule
