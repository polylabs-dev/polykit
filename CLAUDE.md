# PolyKit

**GitHub**: [polylabs-dev/polykit](https://github.com/polylabs-dev/polykit)  
**Platform**: eStream v0.8.1  
**Architecture**: Rust/WASM-first, ESCIR circuits, thin TypeScript DOM binding

## What This Is

PolyKit is the shared framework for all Poly Labs apps (Poly Data, Poly Messenger, Poly Mail, Poly VPN, Poly Pass, Poly OAuth, Poly Mind). It provides the common infrastructure every app needs — identity, crypto, metering, classification, ESLite, console widgets, sanitization — as Rust crates compiled to WASM via the ESCIR codegen pipeline.

## Key Design Principle

**Push everything into Rust/WASM. TypeScript is ONLY a DOM binding layer.**

- All crypto, state management, data transforms, wire protocol, and event processing run in WASM
- ESCIR is the primary client development model (circuits → Rust → WASM → `.escd`)
- TypeScript hooks call WASM exports and render the results — nothing more
- Follows the ESCIR WASM Client Build pipeline (estream-io #550)

## Structure

```
polykit/
├── crates/                  Rust → WASM (the real code)
│   ├── polykit-core/        Identity, PQ crypto, metering, classification
│   ├── polykit-eslite/      ESLite migrations, queries, sync
│   ├── polykit-console/     Widget data pipeline, event bus, RBAC, demo
│   ├── polykit-sanitize/    3-stage PII/PCI/HIPAA/GDPR pipeline
│   └── polykit-wasm/        WASM entry point (wasm-bindgen exports)
├── circuits/                Shared ESCIR circuit definitions
├── packages/react/          @polykit/react — thin TS DOM binding
├── templates/               App scaffolding
└── docs/                    Architecture, getting started, estream feedback
```

## Build

```bash
# Build WASM (via ESCIR pipeline)
estream-dev build-wasm-client --sign key.pem --enforce-budget

# Package as .escd
estream-dev package-escd

# Build thin TS layer
cd packages/react && npm run build
```

## Commit Protocol

Commit to the GitHub issue or epic the work was done under. Do not accumulate large amounts of uncommitted work.

## Developer Language Story (v0.9.1)

eStream supports **7 languages** at full parity: Rust (native), Python (PyO3), TypeScript (WASM), Go (CGo), C++ (FFI), Swift (C bridging), and FastLang (native).

### External Messaging

- Lead with **"7 supported languages"** — developers choose the language they already know
- Position FastLang as **"the shortest path to silicon"** — the easiest way to design for eStream hardware
- **ESCIR (eStream Circuit Intermediate Representation) is strictly internal** — never mention it in external-facing materials, docs, pitches, or marketing. It is an implementation detail of the compiler
- Swift (not Solidity) is the 7th language

### Internal Development

- **FastLang first**: all new circuits and features are authored in FastLang (.fl) first
- **Six-language parity**: every FastLang feature must have equivalent API surface in Rust, Python, TypeScript, Go, C++, and Swift. Do not ship a FastLang-only feature
- Implementation types: FastLang (.fl), Hybrid (FastLang + Rust/RTL), Pure Rust, Pure RTL, Platform (tooling)
- ESCIR operations power the compiler pipeline but are invisible to users

## Cross-Repo Coordination

This repo is part of the [polylabs-dev](https://github.com/polylabs-dev) organization, coordinated through the **AI Toolkit hub** at `toddrooke/ai-toolkit/`.

For cross-repo context, strategic priorities, and the master work queue:
- `toddrooke/ai-toolkit/CLAUDE-CONTEXT.md` — org map and priorities
- `toddrooke/ai-toolkit/scratch/BACKLOG.md` — master backlog
- `toddrooke/ai-toolkit/repos/polylabs-dev.md` — this org's status summary
