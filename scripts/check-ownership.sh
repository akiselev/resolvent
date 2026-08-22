#!/bin/sh
set -eu

production=$(find src -type f -name '*.rs' -print)

if grep -Ein '\.res\b|SemanticModel|ExprId|cadabra-(exact|scalar)|BRep|HalfEdge|Methodus|Solverang|ConstraintGraph|QFunction|TensorProgram' $production; then
    echo 'forbidden consumer vocabulary found in production Resolvent modules' >&2
    exit 1
fi

if grep -REn 'cadabra-(exact|scalar)' Cargo.toml Cargo.lock src; then
    echo 'deleted CADabra algebra facade reference found' >&2
    exit 1
fi
