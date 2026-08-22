# RV0 exact and approximate capability table

| Operation | `f64` | `Interval` | `Rational` | `Real<Rational>` | `SqrtExt<Rational>` | `Dual<S>` |
|---|---|---|---|---|---|---|
| `+ - * /`, order | rounded/approximate | outward enclosure; division across zero returns whole | exact field (zero divisor rejected by checked API) | lazy, exact on decision/force | exact only for compatible root fields; checked | lifted from `S` with chain rule |
| sign/comparison | approximate except exact IEEE zero | certified or `Unknown` | exact | interval filter then exact | exact by algebraic reduction | compares value component |
| integer/rational ingress | nearest representable | enclosing point when finite | exact | exact leaf | exact rational coordinate | lifted from `S` |
| `sqrt` | approximate via `ApproxScalar` | outward enclosure for nonnegative input | `sqrt_exact` only when result is rational | not exposed as transcendental scalar operation | exact represented root | available only when `S: ApproxScalar` |
| `sin/cos/exp/ln` | approximate via `ApproxScalar` | not a public general operation | unavailable | unavailable | unavailable | available only when `S: ApproxScalar` |
| polynomial/resultant/root isolation | coefficients may be admitted exactly from finite IEEE values | filter/evaluation enclosure only | exact coefficients and decisions | not the polynomial coefficient domain | used by specialized radical sign predicates | not a polynomial domain |
| serialization identity | none | none | canonical explicit schema | forbidden runtime DAG state | none yet | none |

`Scalar` means exact-closed field structure, not “has every elementary
function.” `ApproxScalar` is intentionally separate and currently implemented
for `f64`; Resolvent never obtains an exact-looking result by silently routing
an exact value through `f64`.

`Scalar::from_f64` is a proven-input convenience: it is the IEEE identity for
`f64` (including non-finite values) and rejects non-finite exact-`Real` ingress
by panic. Generic untrusted ingress uses `FallibleScalar`, which returns `None`
for non-finite values and zero rational denominators on every supported rung.
