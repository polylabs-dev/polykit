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
