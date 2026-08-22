# RV0 federation baseline

The post-R1 starting point audited by RV0 is:

| Repository | Commit | Role in this gate |
|---|---|---|
| Resolvent | `a916c0307948e6f3a27b2927cee91c2d0edaafb8` | consolidated exact/scalar substrate and RV plan |
| Scientia | `eb8d512020fa4fa6cf99e06012547271525298a1` | scientific-expression owner and exact differentiation consumer |
| CADabra3 | `37ac4d06327a191d50d6413e10612aa1612c4201` | direct exact/filter/root/matrix consumer and geometry-policy owner |

The Resolvent-local gate is the command block in `RV0-EXACT-FOUNDATION.md` plus
`scripts/check-ownership.sh`. Cross-repository gates compile and test Scientia
and CADabra against the resulting Resolvent worktree whenever a consumed public
contract changes. Licensed Parasolid cases remain a CADabra gate and are not
represented as local Resolvent evidence.

This records the baseline used to begin hardening. The commit containing the
hardening itself is intentionally identified by Git history rather than copied
into its own contents.
