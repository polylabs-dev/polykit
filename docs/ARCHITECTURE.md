# PolyKit Architecture

**Version**: 0.1.0  
**Date**: February 2026  
**Platform**: eStream v0.8.1  
**Build Pipeline**: ESCIR → Rust → WASM → .escd (estream-io #550)

---

## Design Principle

**Push everything into Rust/WASM. TypeScript is ONLY a DOM binding layer.**

All crypto, state management, data transforms, wire protocol framing, event processing, RBAC checks, sanitization, and metering run in WASM compiled from ESCIR circuit definitions via the eStream codegen pipeline. TypeScript exists only to mount React components to the DOM and call WASM exports.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│  Browser / React Native                                          │
│                                                                  │
│  @polykit/react (thin TS)                                        │
│  ├── PolyProvider          loads polykit.wasm, inits SPARK       │
│  ├── useWasmSubscription   subscribes via WASM wire client       │
│  ├── useWasmEmit           emits via WASM wire client            │
│  └── WidgetShell           layout + RBAC gate                    │
│                                                                  │
│  TS does: DOM mount, canvas render, browser events               │
│  TS does NOT: crypto, state, data transforms, wire framing       │
├──────────────────────────────────────────────────────────────────┤
│  polykit.wasm (Rust → ESCIR → WASM codegen)                     │
│                                                                  │
│  polykit-core         SPARK identity, PQ crypto, metering        │
│  polykit-eslite       migrations, queries, sync                  │
│  polykit-console      event bus, widget data, demo, RBAC         │
│  polykit-sanitize     3-stage PII/PCI/HIPAA/GDPR pipeline        │
├──────────────────────────────────────────────────────────────────┤
│  eStream Wire Protocol (UDP :5000 / WebTransport :4433)          │
└──────────────────────────────────────────────────────────────────┘
```

## Build Pipeline

Follows the ESCIR WASM Client Build pipeline (estream-io #550):

1. ESCIR circuit definitions (`.escir.yaml`) → Rust codegen
2. `cargo build --target wasm32-unknown-unknown` (LTO, opt-level=z)
3. `wasm-opt -Oz` (dead code elimination)
4. ABI validation (required: `evaluate`; optional: `alloc`, `dealloc`, `circuit_name`, `circuit_version`)
5. Size budget check (≤128 KB/circuit, ≤512 KB total, ≤4 MB linear memory)
6. ML-DSA-87 signing → `.wasm.sig`
7. `.escd` packaging (manifest.json + .wasm + .wasm.sig)

## Crates

| Crate | Purpose | Size Target |
|-------|---------|-------------|
| polykit-core | SPARK identity, ML-DSA-87/ML-KEM-1024, metering, classification | ≤64 KB |
| polykit-eslite | Migration runner, schema DSL, query engine, sync | ≤32 KB |
| polykit-console | Event bus, widget data pipeline, demo fixtures, RBAC | ≤32 KB |
| polykit-sanitize | 3-stage compliance pipeline (PII detect, transform, audit) | ≤24 KB |
| polykit-wasm | WASM entry point (wasm-bindgen exports) | Overhead only |

## ESCIR Circuit Templates

| Circuit | Purpose | Target |
|---------|---------|--------|
| polykit-identity | SPARK auth + HKDF key derivation | wasm-client |
| polykit-metering | 8-dimension resource metering | wasm-client |
| polykit-telemetry | StreamSight observability pipeline | wasm-client |
| polykit-rate-limiter | FIFO rate limiter with backpressure | wasm-client |
| polykit-sanitize | 3-stage compliance sanitization | wasm-client |
| polykit-eslm-classify | Generic ESLM auto-classification | wasm-client |

## How Apps Use PolyKit

Apps depend on PolyKit Rust crates and extend with domain-specific ESCIR circuits:

```toml
# App Cargo.toml
[dependencies]
polykit-core = { git = "https://github.com/polylabs-dev/polykit" }
polykit-eslite = { git = "https://github.com/polylabs-dev/polykit" }
```

```yaml
# App circuit extends shared template
escir: "0.8.1"
name: polymail-inbox
extends: polykit-metering
imports:
  - polykit-core::identity
  - polykit-eslite::query
```

```tsx
// App TS — minimal DOM binding
import { PolyProvider } from '@polykit/react';

export default function App() {
  return (
    <PolyProvider wasm="/pkg/polymail.wasm" hkdfContext="poly-mail-v1">
      <WidgetGrid />
    </PolyProvider>
  );
}
```

## Related Documents

| Document | Purpose |
|----------|---------|
| [ESTREAM_FEEDBACK.md](ESTREAM_FEEDBACK.md) | Running feedback on eStream platform |
| [estream-io #550](https://github.com/polyquantum/estream-io/issues/550) | ESCIR WASM Client Build pipeline |
| [ESCIR_WASM_CLIENT_SPEC.md](https://github.com/polyquantum/estream-io/blob/main/specs/architecture/ESCIR_WASM_CLIENT_SPEC.md) | Full WASM client spec |
