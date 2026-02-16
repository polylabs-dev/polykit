/**
 * @polykit/react — Thin DOM binding layer for PolyKit WASM
 *
 * This package provides ONLY:
 * - React context provider (loads WASM)
 * - Hooks that call WASM exports (no business logic in TS)
 * - DOM components (WidgetShell, SparkRenderer)
 *
 * All crypto, data transforms, state management, wire protocol,
 * and event processing run in WASM.
 */

// Provider
export { PolyProvider, usePolyContext } from './PolyProvider';

// Hooks (thin wrappers over WASM)
export {
  useWasmSubscription,
  useWasmEmit,
  useWasmWidgetData,
  useEmitWidgetEvent,
  useSanitize,
} from './hooks';

// Components (DOM-only)
export { WidgetShell } from './WidgetShell';

// WASM bridge (for advanced use)
export { loadWasm, getWasm, parseWasmResponse } from './wasm-bridge';
export type { PolykitWasm } from './wasm-bridge';
