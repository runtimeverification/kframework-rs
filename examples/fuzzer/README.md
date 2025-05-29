# Imp fuzzing with kframework_ffi

This is an example of a fuzzing target for a hashing function that uses the kllvm execution engine for the Imp semantics.

## Requirements

- An installation of the [kframework](https://github.com/runtimeverification/k)
- An installation of [rustup](https://rustup.rs/)
- Cargo installed [honggfuzz](https://github.com/rust-fuzz/honggfuzz-rs)

NOTE: If you are using a `kup` installed version of K, then you are very likely to run into linking errors between the nix libraries and your installation of `rustup`. This should be able to get resolved by a proper derivation, but no such thing exists at the time of writing.

## Setup

```
$ cd k-semantics
$ kompile --llvm-kompile-type c fuzz.k
$ export LD_LIBRARY_PATH=$(realpath fuzz-kompiled)
$ cd ..
$ cargo hfuzz run fuzzer
```

## Explanation

The `FUZZ` semantics module extends Imp with an `Assert` statement which rewrites the entire configuration into a failing symbol when it doesn't pass. It also has a success symbol which it will rewrite the configuration to when execution is finished (ie. the <k> cell is empty).

There is an `init_fuzz` utility symbol which takes two integer parameters and then rewrites to an Imp program that performs a hash over those integers. The failure case for this program is arbitrarily if the hash ends up being a very low number.

On the rust side, the fuzzing target uses the random input to construct this `init_fuzz` symbol and the initial configuration in the form of a kore string. Then it uses the foreign functions to build that configuration in kllvm's interned representation, execute it, and retrieve the resulting configuration as another kore string. Then it parses the kore string and checks for the failure or success case.
