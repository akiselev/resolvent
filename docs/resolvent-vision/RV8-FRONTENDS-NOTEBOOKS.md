# RV8 - Language, Kernel Protocol and Notebooks

## Goal

Make the full Resolvent kernel usable interactively and remotely without letting any frontend define the CAS semantics.

RV8 starts early after RV1's canonical wire identity. It progresses from parser/CLI to a transport-neutral kernel protocol, Jupyter, foreign bindings and finally a native notebook. Algorithm breadth continues in parallel through RV5-RV7.

## Principles

- Rust library API remains authoritative.
- The kernel protocol transports structured values/plans/receipts, not terminal text only.
- Jupyter is an adapter, not the internal protocol.
- Notebook documents are replayable source plus environment metadata, not opaque serialized process state.
- Sequential cells are the default; reactive cells are explicitly pure/declared.
- Local and remote kernels use the same protocol semantics.
- Frontend cancellation/timeouts do not redefine deterministic mathematical results.

## Work packages

### RV8-A - Surface syntax and parser

#### RV8-A1 - Minimal mathematical syntax

Define a small language over RV1 terms and RV2 values:

- exact integer/rational/decimal literals;
- explicit approximate literals/precision forms;
- symbols and namespaced names;
- arithmetic and relations;
- function calls;
- lists/tuples/maps;
- indexing;
- assignments/definitions;
- assumptions;
- rules/pattern syntax once RV4 is available;
- package imports;
- explicit algorithm/precision/budget options.

Do not begin with a large general-purpose programming language. Add control constructs only when concrete notebook/package use requires them.

#### RV8-A2 - Lossless parser/formatter contract

The parser produces RV1 term/session commands without passing exact literals through `f64`.

Provide:

- source spans;
- structured diagnostics;
- deterministic formatter;
- syntax-only parse mode;
- canonical/debug term inspection;
- source-to-term origin sidecars.

Formatting is presentation canonicalization, not semantic evaluation.

### RV8-B - CLI and REPL

#### RV8-B1 - Batch CLI

Support commands such as:

```text
resolvent eval 'factor(x^4 - 1)'
resolvent parse file.rv
resolvent check file.rv
resolvent plan file.rv
resolvent explain <receipt-or-plan>
resolvent verify <certificate>
resolvent render --format latex|mathml|json
```

Exact command shape may change, but every important kernel artifact should be inspectable without a GUI.

#### RV8-B2 - Interactive REPL

Provide:

- persistent session definitions/assumptions;
- multiline editing;
- completion/inspection;
- history;
- rich text/LaTeX-capable terminal fallbacks where supported;
- interrupt/cancellation;
- plan/receipt inspection.

The REPL uses the same session API as remote kernels.

### RV8-C - Transport-neutral kernel protocol

#### RV8-C1 - Protocol schema

Define versioned request/response messages for:

- create/close session;
- evaluate command/term;
- cancel request;
- completion;
- inspect/documentation;
- render/format;
- plan/explain;
- receipt/certificate verification;
- artifact put/get/reference;
- session snapshot/replay metadata;
- package/environment inspection;
- progress/events for long operations.

Messages carry stable request IDs, session IDs and RV1/RV3 artifact digests. Large payloads may be out-of-band content-addressed artifacts.

#### RV8-C2 - In-process transport

Implement the protocol against the Rust kernel in-process first. This validates message semantics without networking/process complexity.

#### RV8-C3 - Local process transport

Add framed IPC or stdio/socket transport for separate frontend/kernel processes.

Requirements:

- protocol version negotiation;
- structured cancellation;
- crash detection;
- no arbitrary frontend object pointers/handles on the wire;
- deterministic replay of completed mathematical requests.

#### RV8-C4 - Remote transport

Add authenticated remote transport only after local process semantics stabilize. Transport/security code is isolated from algebra crates.

### RV8-D - Rich display model

Define structured display values independent of Jupyter MIME specifics:

- plain text;
- LaTeX;
- MathML;
- HTML fragments where safe;
- tables/data grids;
- SVG/vector graphics;
- raster images;
- plots;
- 2D/3D scene descriptions;
- tree/term inspector data;
- plan/receipt/certificate inspector data.

Adapters choose their native presentation (MIME bundles for Jupyter, widgets for native UI).

### RV8-E - Jupyter kernel

Build a Jupyter adapter over the Resolvent protocol.

Support:

- execute requests;
- completion;
- inspection/help;
- interrupts;
- rich display bundles;
- error diagnostics;
- kernel metadata/version;
- notebook persistence through standard `.ipynb` source/output cells.

Jupyter becomes the first serious interactive frontend because it immediately unlocks JupyterLab, VS Code and remote notebook workflows.

Acceptance notebooks exercise:

- exact arithmetic;
- polynomial/domain operations;
- assumptions/rewrite;
- algorithm plan inspection;
- receipts/certificate verification;
- at least one Scientia extension command once that integration exists.

### RV8-F - Foreign language bindings

#### RV8-F1 - Python

Provide Python objects for:

- session;
- term;
- domain/element;
- outcome;
- plan;
- receipt/certificate.

Avoid hiding everything behind `eval(str)`. Structured construction/inspection must be available.

#### RV8-F2 - C ABI

Expose a stable handle-based C ABI suitable for other languages and embedded applications.

No Rust trait object crosses the ABI. All errors/outcomes are explicit.

#### RV8-F3 - JavaScript/WASM

Support browser/local WASM for pure-kernel capabilities that fit the dependency/size envelope. Heavy native providers are not required in WASM.

### RV8-G - Native notebook document format

Define `.rnb` (working name) as a replayable document containing:

- stable notebook/cell IDs;
- source cells;
- Markdown/math cells;
- cell type/metadata;
- explicit pure/reactive declarations;
- dependency metadata where known;
- package/environment/kernel-profile manifest;
- notebook-level assumptions/settings;
- optional content-addressed references to outputs/receipts/certificates;
- schema/version.

Do not make arbitrary live session memory the source of truth.

Support import/export with `.ipynb` for overlapping cell/output concepts.

### RV8-H - Native notebook UI

Build only after the protocol/Jupyter path is exercised.

Core surfaces:

- code/math/Markdown cells;
- typeset input/output;
- table/plot/scene cells;
- term tree inspector;
- domain/coercion inspector;
- active assumption/definition browser;
- algorithm plan inspector showing chosen algorithm, applicability reasons, fallbacks, budgets and provider;
- receipt/certificate viewer;
- package/docs browser;
- execution/dependency history;
- local/remote kernel selector.

The algorithm inspector is a first-class differentiator: CAS users and agents should be able to see not only the answer but how Resolvent decided to compute it.

### RV8-I - Reactive pure cells

Add Pluto-style reactive execution only for explicitly pure cells/sections.

Requirements:

- dependency graph derived from declared/analysed symbol dependencies;
- no hidden mutation inside pure cells;
- cache key includes source, dependencies, assumptions, package environment and relevant plan/provider identities;
- expensive cells have policy controls before automatic rerun;
- ordinary sequential cells remain available for intentionally stateful workflows.

### RV8-J - Export and reproducibility

Support export to:

- HTML;
- Markdown;
- `.ipynb`;
- PDF through a renderer pipeline;
- script/source form;
- structured bundle with referenced receipts/certificates.

A reproducibility report lists:

- Resolvent version/commit;
- package versions;
- providers;
- assumptions/settings;
- cell execution order or reactive graph;
- artifact digests.

## Extension model

Notebook/CLI extension packages may expose higher-level commands from:

- Scientia;
- CADabra;
- Methodus;
- Solverang;
- Sinbad.

The result must retain extension/provider identity. Frontend convenience does not move semantic ownership into Resolvent.

Examples:

- a Solverang extension may construct/inspect a constraint solve while Solverang remains authority for constraint semantics and Methodus supplies numerical solving;
- a CADabra extension may display a B-rep/constraint-driven geometry while CADabra remains geometry authority;
- a Scientia extension may compile a `.res` model and expose embedded Resolvent scalar terms.

## Exit gate

RV8 exits when:

- a stable transport-neutral kernel protocol exists;
- CLI and REPL use the same session/evaluation contracts as remote clients;
- Jupyter is a first-class usable frontend;
- Python and C APIs provide structured access rather than string-only evaluation;
- native notebook documents are replayable and environment-aware;
- native UI exposes algebra/domain/plan/evidence inspection;
- optional pure reactive cells cannot silently rerun stateful work;
- frontend code is not required by the core CAS library.

## Parallelism

A parser/CLI lane can start as RV1 stabilizes. Protocol schema/in-process transport can run in parallel with later RV2/RV3 work once term/outcome identity is known. Jupyter follows the protocol. Native UI starts last. Binding lanes can fan out after protocol/dynamic-value APIs settle.

## Non-goals

- making Jupyter protocol the internal kernel protocol;
- serializing opaque process state as notebook truth;
- requiring network/runtime dependencies in core algebra crates;
- duplicating Scientia/CADabra/Methodus/Solverang/Sinbad semantics in the frontend;
- blocking CAS algorithm breadth on native UI completion.