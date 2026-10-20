# Deliberate deviations from CalcpadCE

The port reproduces CalcpadCE output exactly unless a behaviour is listed here. Each entry
names the behaviour, why it differs, the test that pins it, and the conformance stubs
(`*.html.stub`) whose expected output it changes.

## D1. A failing `#if` / `#else if` condition runs no branch

**CalcpadCE:** when the condition expression throws (for example a misspelled variable,
`#if v > v_typo`), `ExpressionParser.ParseTokens` catches the error and `ParseLine` then
calls `Condition.Check(_parser.Result)`. `MathParser.Parse` reset the result to zero, so
the condition is treated as false and the `#else` branch runs. In calculation mode the
condition line is not part of the output (`isOutput` is false when the keyword length is
non-zero), so the error is not shown at all: the worksheet silently takes the `#else`
branch. With

```
#if v > v_typo
'NOT OK
#else
'OK
#end if
```

CalcpadCE prints `OK` and nothing else.

**calcpad-rs:** the block is left unresolved. Neither the branch of the failing condition
nor any later `#else if` / `#else` of the same chain runs (`Condition.check_failed` marks
the chain as found and false), and an error paragraph is written in place of the block:

```
Error in "v > v_typo" on line [n]: Undefined variable or units: "v_typo". The condition
could not be evaluated, so the block is unresolved and none of its branches were run.
```

Valid conditions, `#while` conditions and condition errors outside calculation mode are
unchanged.

**Tests:** `calcpad-core/tests/deviations.rs`
(`d1_failed_if_condition_runs_neither_branch`,
`d1_failed_else_if_condition_skips_the_rest_of_the_chain`,
`valid_conditions_are_unchanged`).

**Stubs changed:** none of the 98 in-scope worksheets. Two out-of-scope worksheets hit
the path, both as a consequence of an earlier out-of-scope failure:

| Worksheet | Cause |
|---|---|
| `Examples/Engineering/Numerical/Rayleigh Quotient Iteration.cpd` | `λ` is a vector that phase 1 cannot compute, so `#if abs(λ - λ_0) < 10^-3` fails |
| `Examples/Isaiah/UI Mode Test.cpd` | `useSteel` is defined by an unsupported `#UI` line, so `#if useSteel ≡ 1` fails |

When vectors and `#UI` are ported these conditions evaluate normally and the stubs are
expected to match again.

## Not deviations (scope and known differences)

These are not deliberate behaviour changes, but they can make output differ:

* **Out of scope for phase 1** (vectors, matrices, HP types, `$Plot`/`$Map`, `#UI`,
  `#read`/`#write`/`#append`, markdown, Word/OpenXml): the engine reports
  `... is not supported by calcpad-rs yet (out of scope for phase 1).` as an ordinary
  expression error and records the feature so the harness classifies the worksheet as
  OUT-OF-SCOPE.
* **Floating point library:** `f64::sin`, `exp`, `pow` and friends come from the platform
  libm (macOS here), CalcpadCE uses the .NET runtime's. Last digit differences are
  absorbed by the harness except where a stub prints an exact integer (the reconciliation
  ignores integers), which is the cause of the only in-scope failure,
  `Tests/Others/Integration tests.cpd` (`I_191` prints `-1.000000000000001` instead of
  `-1` at `#round 15`).
* **Custom function result cache:** `CustomFunction1/2` memoise results per argument and
  clear the cache when a dependency changes. The port always evaluates. The cache is
  transparent in CalcpadCE (it is invalidated through `OnChange`), so no output differs.
