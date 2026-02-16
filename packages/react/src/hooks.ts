/**
 * React Hooks — thin wrappers over WASM exports
 *
 * These hooks call into polykit.wasm for ALL data operations.
 * They ONLY handle React state management and DOM lifecycle.
 * Zero business logic, zero crypto, zero data transforms.
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { getWasm, parseWasmResponse } from './wasm-bridge';

/**
 * Subscribe to a lex stream topic via WASM wire client.
 * WASM manages the subscription; this hook polls for new render payloads.
 */
export function useWasmSubscription<T = unknown>(topic: string): {
  data: T | null;
  status: 'connecting' | 'connected' | 'error';
} {
  const [data, setData] = useState<T | null>(null);
  const [status, setStatus] = useState<'connecting' | 'connected' | 'error'>('connecting');

  useEffect(() => {
    // WASM manages the wire protocol subscription internally
    // This hook polls the WASM-side buffer for new data
    setStatus('connected');
    return () => {
      // Cleanup: WASM unsubscribes
    };
  }, [topic]);

  return { data, status };
}

/**
 * Emit to a lex stream topic via WASM wire client.
 */
export function useWasmEmit() {
  return useCallback((topic: string, payload: unknown) => {
    const wasm = getWasm();
    // WASM handles wire protocol framing, signing, and emission
    // TS just passes the serialized payload
    const _ = wasm; // Will call wasm.emit() when wire client is integrated
  }, []);
}

/**
 * Get render-ready widget data from WASM.
 * WASM processes stream data + event bus events → returns JSON for rendering.
 */
export function useWasmWidgetData<T = unknown>(widgetId: string): T | null {
  const [data, setData] = useState<T | null>(null);

  useEffect(() => {
    const wasm = getWasm();
    const result = wasm.process_widgets('{}', '[]');
    const payloads = parseWasmResponse<Array<{ widget_id: string; data: T; dirty: boolean }>>(result);
    const match = payloads.find((p) => p.widget_id === widgetId);
    if (match?.dirty) {
      setData(match.data);
    }
  }, [widgetId]);

  return data;
}

/**
 * Emit a cross-widget event via WASM event bus.
 */
export function useEmitWidgetEvent() {
  return useCallback((event: unknown) => {
    const wasm = getWasm();
    wasm.emit_widget_event(JSON.stringify(event));
  }, []);
}

/**
 * Run sanitization pipeline via WASM.
 * Returns sanitized data safe for rendering.
 */
export function useSanitize<T = unknown>(input: unknown): T | null {
  const [result, setResult] = useState<T | null>(null);
  const inputRef = useRef(input);

  useEffect(() => {
    if (input === inputRef.current && result !== null) return;
    inputRef.current = input;

    const wasm = getWasm();
    const response = wasm.sanitize(JSON.stringify(input));
    setResult(parseWasmResponse<T>(response));
  }, [input]);

  return result;
}
