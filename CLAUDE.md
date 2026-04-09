# QKit

**GitHub**: [polylabs-dev/qkit](https://github.com/polylabs-dev/qkit)
**Platform**: eStream v0.22.0
**Architecture**: 100% FastLang, FLIR codegen pipeline

## What This Is

QKit is the shared framework for all PolyQ Labs apps (Q Files, Q Messenger, Q Mail, Q VPN, Q Pass, Q OAuth, Q Mind, Q Git). It provides the common infrastructure every app needs — identity, crypto, metering, classification, ESLite, console widgets, sanitization — as FastLang circuits compiled via the FLIR codegen pipeline (FL → FLIR → Rust/WASM).

## Key Design Principle

**100% FastLang. No hand-written Rust.** All crypto, state management, data transforms, wire protocol, and event processing are authored in FastLang and compiled via FLIR codegen. TypeScript is ONLY a DOM binding layer.

> **Note**: `crates/` are being slimmed to near-zero as FLIR codegen replaces the remaining hand-written Rust. All new work is FastLang-only.

## v0.22.0 Updates

- **8 new composition circuits** added for cross-product infrastructure (zero-linkage bridge, blinded billing, classified fusion, CRDT sync, media stream, blind relay, regional compliance US/EU)
- Total circuit count: 30 (22 original + 8 new composition circuits)
- FLIR codegen now handles all compilation

## Structure

```
qkit/
├── circuits/                FLIR circuit definitions (the real code)
│   ├── fl/                  FastLang source (.fl files)
│   └── ...
├── crates/                  Legacy Rust (being slimmed to near-zero via FL codegen)
│   ├── qkit-core/        Identity, PQ crypto, metering, classification
│   ├── qkit-eslite/      ESLite migrations, queries, sync
│   ├── qkit-console/     Widget data pipeline, event bus, RBAC, demo
│   ├── qkit-sanitize/    3-stage PII/PCI/HIPAA/GDPR pipeline
│   └── qkit-wasm/        WASM entry point (wasm-bindgen exports)
├── packages/react/          @qkit/react — thin TS DOM binding
├── templates/               App scaffolding
└── docs/                    Architecture, getting started
```

## Build

```bash
# Build via FLIR pipeline
estream-dev build-wasm-client --sign key.pem --enforce-budget

# Package as .escd
estream-dev package-escd

# Build thin TS layer
cd packages/react && npm run build
```

## Commit Protocol

Commit to the GitHub issue or epic the work was done under. Do not accumulate large amounts of uncommitted work.
