/**
 * WASM Bridge — typed FFI between React and polykit.wasm
 *
 * This is the ONLY file that calls wasm-bindgen exports directly.
 * All other TS files in this package call through this bridge.
 *
 * Design rule: This bridge passes serialized JSON strings across the
 * boundary. WASM owns all data transforms. TS only renders.
 */

export interface PolykitWasm {
  init_app(appId: string, hkdfContext: string, lexNamespace: string, demoMode: boolean): string;
  derive_identity(masterSeed: Uint8Array, hkdfContext: string, lexNamespace: string): string;
  run_migrations(migrationsJson: string): string;
  query(table: string, filterJson: string): string;
  process_widgets(streamDataJson: string, eventsJson: string): string;
  emit_widget_event(eventJson: string): string;
  sanitize(inputJson: string): string;
  classify(path: string, policyJson: string): string;
  check_metering_limits(currentJson: string, limitsJson: string): string;
  evaluate(contextPtr: number): number;
  circuit_name(): string;
  circuit_version(): string;
}

let wasmInstance: PolykitWasm | null = null;

/**
 * Load and initialize the PolyKit WASM module.
 * Called once from PolyProvider on mount.
 */
export async function loadWasm(wasmUrl: string): Promise<PolykitWasm> {
  if (wasmInstance) return wasmInstance;

  const module = await import(/* webpackIgnore: true */ wasmUrl);
  await module.default();

  wasmInstance = module as unknown as PolykitWasm;
  return wasmInstance;
}

/**
 * Get the loaded WASM instance. Throws if not yet loaded.
 */
export function getWasm(): PolykitWasm {
  if (!wasmInstance) {
    throw new Error('PolyKit WASM not loaded. Wrap your app in <PolyProvider>.');
  }
  return wasmInstance;
}

/**
 * Parse a JSON response from WASM, handling errors.
 */
export function parseWasmResponse<T>(json: string): T {
  const parsed = JSON.parse(json);
  if (parsed.error) {
    throw new Error(`PolyKit WASM error: ${parsed.error}`);
  }
  return parsed as T;
}
