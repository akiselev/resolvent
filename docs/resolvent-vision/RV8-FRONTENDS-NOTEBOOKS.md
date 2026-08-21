# RV8 - Language, Kernel Protocol and Notebooks

## Goal

Make the Resolvent kernel usable interactively and remotely without letting any frontend define CAS semantics.

RV8 starts early, but **different RV8 lanes have different prerequisites**. RV1 structural Term identity is enough for parser/formatter/simple-CLI prototypes. It is not enough to freeze a protocol that transports dynamic values, plans, outcomes and receipts.

Algorithm breadth continues in parallel through RV5-RV7.

## Principles

- Rust library API remains authoritative.
- The kernel protocol transports structured values/plans/receipts, not terminal text only.
- Jupyter is an adapter, not the internal protocol.
- Notebook documents are replayable source plus environment metadata, not opaque serialized process state.
- Sequential cells are the default; reactive cells are explicitly pure/declared.
- Local and remote kernels use the same protocol semantics.
- Frontend cancellation/timeouts do not redefine deterministic mathematical results.
- Protocol prototyping may precede protocol stability; compatibility promises follow the schemas they transport.

## Prerequisite ladder

| RV8 surface | Minimum prerequisite |
|---|---|
| structural parser/formatter | RV1 atom/Term syntax and structural identity |
| simple batch CLI over existing typed Rust APIs | RV1 plus the operations it invokes |
| dynamic domain/value inspection | corresponding RV2 `Domain`/`Element` schemas |
| plan/explain/receipt commands | corresponding RV3 request/outcome/plan/receipt schemas |
| stateful definitions/assumptions/rules | relevant RV4 session/evaluation semantics |
| stable transport-neutral protocol | coherent initial RV1 Term + RV2 dynamic value + RV3 outcome/plan/receipt schemas |
| Jupyter compatibility promise | stable protocol subset plus rich-display schema |
| native notebook | exercised CLI/protocol/Jupyter semantics |

No RV8 work package should cite only a phase number when a narrower typed prerequisite can be named.

## Work packages

### RV8-A - Surface syntax and parser

#### RV8-A1 - Minimal mathematical syntax

May start as RV1 structural syntax stabilizes.

Define a small language over available RV1 Terms and, incrementally, RV2 values:

- exact integer/rational/decimal literals;
- explicit approximate literals/precision forms;
- symbols and namespaced names;
- arithmetic and relations;
- function calls;
- lists/tuples/maps;
- indexing;
- assignments/definitions once RV4 defines their semantics;
- assumptions once RV4 defines their semantics;
- rules/pattern syntax once RV4 is available;
- package imports once package semantics exist;
- explicit algorithm/precision/budget options once RV3 provides those request fields.

Do not begin with a large general-purpose programming language. Add control constructs only when concrete notebook/package use requires them.

#### RV8-A2 - Lossless parser/formatter contract

The Resolvent parser produces RV1 Terms/session commands without passing exact literals through `f64`.

Provide:

- source spans;
- structured diagnostics;
- deterministic formatter;
- syntax-only parse mode;
- canonical/debug structural Term inspection;
- source-to-Term origin sidecars.

Formatting is presentation canonicalization, not semantic evaluation.

This parser does not replace Scientia's `.res` parser. A Scientia notebook extension invokes Scientia for `.res` semantics.

### RV8-B - CLI and REPL

#### RV8-B1 - Batch CLI

A narrow CLI can start early over whichever typed operations have landed:

```text
resolvent eval 'factor(x^4 - 1)'
resolvent parse file.rv
resolvent check file.rv
resolvent plan file.rv
resolvent explain <receipt-or-plan>
resolvent verify <certificate>
resolvent render --format latex|mathml|json
```

Commands appear only when the underlying semantic contract exists. For example, `plan`/`explain` stability follows RV3 rather than being invented independently in the CLI.

#### RV8-B2 - Interactive REPL

Provide incrementally:

- persistent session definitions/assumptions after RV4;
- multiline editing;
- completion/inspection;
- history;
- rich text/LaTeX-capable terminal fallbacks where supported;
- interrupt/cancellation;
- plan/receipt inspection after RV3.

The REPL uses the same session API as remote kernels once that API is stable.

### RV8-C - Transport-neutral kernel protocol

#### RV8-C0 - Prototype envelope

Before protocol stability, prototype request IDs, framing, errors and cancellation using structural Terms and mock/draft dynamic payloads.

The prototype is explicitly unstable and exists to discover mistakes before public schemas freeze.

#### RV8-C1 - Stable protocol schema

Freeze versioned request/response messages only after the initial RV1/RV2/RV3 schemas they carry are coherent.

Messages cover:

- create/close session;
- evaluate command/Term;
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

Messages carry stable request IDs, session IDs and semantic artifact digests. They never put arena-local handles on the wire.

Large payloads may use content-addressed references. When durable cross-process/cross-repository storage and lineage are required, an optional Artifactum adapter is preferred to creating a second general artifact store.

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
- extension-defined 2-D/3-D display payloads;
- tree/Term inspector data;
- domain/coercion inspector data;
- plan/receipt/certificate inspector data.

Resolvent owns the generic rich-display envelope. CADabra/Astoria/other extensions own domain-specific scene semantics rather than moving geometry/world models into the CAS core.

### RV8-E - Jupyter kernel

Build a Jupyter adapter over the **stable subset** of the Resolvent protocol.

Support:

- execute requests;
- completion;
- inspection/help;
- interrupts;
- rich display bundles;
- error diagnostics;
- kernel metadata/version;
- notebook persistence through standard `.ipynb` source/output cells.

Jupyter becomes the first serious interactive frontend because it unlocks JupyterLab, VS Code and remote notebook workflows without forcing the native notebook to define kernel semantics.

Acceptance notebooks exercise only capabilities whose underlying contracts have landed, eventually including:

- exact arithmetic;
- polynomial/domain operations;
- assumptions/rewrite;
- algorithm plan inspection;
- receipts/certificate verification;
- at least one Scientia extension command.

### RV8-F - Foreign language bindings

#### RV8-F1 - Python

Provide structured Python objects for landed concepts:

- session;
- Term;
- domain/element;
- outcome;
- plan;
- receipt/certificate.

Avoid hiding everything behind `eval(str)`.

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
- table/plot/extension-display cells;
- Term tree inspector;
- domain/coercion inspector;
- active assumption/definition browser;
- algorithm plan inspector showing chosen algorithm, applicability reasons, fallbacks, budgets and provider;
- receipt/certificate viewer;
- package/docs browser;
- execution/dependency history;
- local/remote kernel selector.

The algorithm inspector is a first-class differentiator: users and agents should be able to see not only the answer but how Resolvent decided to compute it.

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
- a Scientia extension may compile a `.res` model and display Resolvent algebra projections without making Resolvent Terms Scientia's canonical scientific expression identity.

## Exit gate

RV8 reaches mature status when:

- a stable transport-neutral kernel protocol exists over stable initial Term/value/outcome schemas;
- CLI and REPL use the same session/evaluation contracts as remote clients for their stable features;
- Jupyter is a first-class usable frontend;
- Python and C APIs provide structured access rather than string-only evaluation;
- native notebook documents are replayable and environment-aware;
- native UI exposes algebra/domain/plan/evidence inspection;
- optional pure reactive cells cannot silently rerun stateful work;
- frontend code is not required by the core CAS library.

## Parallelism

RV8-A parser/formatter and a narrow RV8-B1 CLI can start with RV1. RV8-C0 protocol prototyping can also start early. Stable RV8-C1/C2 semantics wait for the initial RV2 dynamic-value and RV3 outcome/plan/receipt contracts. Jupyter follows that stable subset. Stateful REPL features wait for RV4. Binding lanes fan out as the dynamic/value/protocol surfaces settle. Native UI starts last.

## Non-goals

- treating RV1 completion alone as a full protocol freeze;
- making Jupyter protocol the internal kernel protocol;
- serializing opaque process state as notebook truth;
- requiring network/runtime dependencies in core algebra crates;
- duplicating Scientia/CADabra/Methodus/Solverang/Sinbad semantics in the frontend;
- duplicating Artifactum for durable artifact lifecycle;
- blocking CAS algorithm breadth on native UI completion.
