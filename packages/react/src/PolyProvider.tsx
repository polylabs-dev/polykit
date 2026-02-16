/**
 * PolyProvider — app context provider
 *
 * Loads polykit.wasm, initializes SPARK identity, and provides
 * the WASM bridge to all child components.
 *
 * This is the ONLY component that interacts with WASM loading.
 */

import React, { createContext, useContext, useEffect, useState } from 'react';
import { loadWasm, parseWasmResponse, type PolykitWasm } from './wasm-bridge';

interface PolyContextValue {
  wasm: PolykitWasm | null;
  appId: string;
  lexNamespace: string;
  ready: boolean;
  error: string | null;
}

const PolyContext = createContext<PolyContextValue>({
  wasm: null,
  appId: '',
  lexNamespace: '',
  ready: false,
  error: null,
});

export function usePolyContext(): PolyContextValue {
  return useContext(PolyContext);
}

interface PolyProviderProps {
  /** URL to the app's .wasm file (includes polykit + app-specific circuits) */
  wasm: string;
  /** HKDF derivation context (e.g., "poly-data-v1") */
  hkdfContext: string;
  /** App identifier (e.g., "polydata") */
  appId?: string;
  /** Lex namespace (e.g., "polylabs.data") */
  lexNamespace?: string;
  /** Enable demo mode (?demo=true equivalent) */
  demo?: boolean;
  children: React.ReactNode;
}

export function PolyProvider({
  wasm: wasmUrl,
  hkdfContext,
  appId = '',
  lexNamespace = '',
  demo = false,
  children,
}: PolyProviderProps) {
  const [state, setState] = useState<PolyContextValue>({
    wasm: null,
    appId,
    lexNamespace,
    ready: false,
    error: null,
  });

  useEffect(() => {
    let cancelled = false;

    async function init() {
      try {
        const wasmModule = await loadWasm(wasmUrl);

        if (cancelled) return;

        // Initialize app in WASM
        const resultJson = wasmModule.init_app(appId, hkdfContext, lexNamespace, demo);
        const result = parseWasmResponse<{ app_id: string; lex_namespace: string; status: string }>(resultJson);

        setState({
          wasm: wasmModule,
          appId: result.app_id,
          lexNamespace: result.lex_namespace,
          ready: true,
          error: null,
        });
      } catch (err) {
        if (cancelled) return;
        setState((prev) => ({
          ...prev,
          error: err instanceof Error ? err.message : 'Failed to load WASM',
        }));
      }
    }

    init();
    return () => { cancelled = true; };
  }, [wasmUrl, hkdfContext, appId, lexNamespace, demo]);

  if (state.error) {
    return <div className="polykit-error">PolyKit initialization failed: {state.error}</div>;
  }

  if (!state.ready) {
    return <div className="polykit-loading">Loading PolyKit...</div>;
  }

  return <PolyContext.Provider value={state}>{children}</PolyContext.Provider>;
}
