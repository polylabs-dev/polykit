/**
 * React Hooks — thin wrappers over WASM exports
 *
 * These hooks call into qkit.wasm for ALL data operations.
 * They ONLY handle React state management and DOM lifecycle.
 * Zero business logic, zero crypto, zero data transforms.
 *
 * v0.11.0: WebTransport (QUIC/HTTP3), RBAC, LexStream hooks
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { getWasm, parseWasmResponse } from './wasm-bridge';

/**
 * Subscribe to a lex stream topic via WASM WebTransport (QUIC/HTTP3 datagrams).
 * WASM manages the WebTransport session on port 4433; this hook
 * receives typed payloads as they arrive via datagram.
 */
export function useWasmSubscription<T = unknown>(topic: string): {
  data: T | null;
  status: 'connecting' | 'connected' | 'error';
} {
  const [data, setData] = useState<T | null>(null);
  const [status, setStatus] = useState<'connecting' | 'connected' | 'error'>('connecting');

  useEffect(() => {
    const wasm = getWasm();
    const handle = wasm.subscribe_webtransport(topic, (payload: Uint8Array) => {
      setData(parseWasmResponse<T>(payload));
      setStatus('connected');
    });
    setStatus('connecting');
    return () => {
      wasm.unsubscribe_webtransport(handle);
    };
  }, [topic]);

  return { data, status };
}

/**
 * Emit to a lex stream topic via WASM WebTransport datagrams.
 */
export function useWasmEmit() {
  return useCallback((topic: string, payload: unknown) => {
    const wasm = getWasm();
    wasm.emit_webtransport(topic, JSON.stringify(payload));
  }, []);
}

/**
 * RBAC-aware hook backed by WASM. Fail-closed: if WASM cannot verify
 * the role, access is denied. Returns the resolved permission set.
 */
export function useRbac(requiredRole: string): {
  allowed: boolean;
  role: string | null;
  loading: boolean;
} {
  const [state, setState] = useState<{
    allowed: boolean;
    role: string | null;
    loading: boolean;
  }>({ allowed: false, role: null, loading: true });

  useEffect(() => {
    const wasm = getWasm();
    try {
      const result = parseWasmResponse<{ allowed: boolean; role: string }>(
        wasm.check_rbac(requiredRole)
      );
      setState({ allowed: result.allowed, role: result.role, loading: false });
    } catch {
      setState({ allowed: false, role: null, loading: false });
    }
  }, [requiredRole]);

  return state;
}

/**
 * Typed lex stream subscription. Receives structured events from a
 * specific lex namespace path via the WASM wire client.
 */
export function useLexStream<T = unknown>(
  lexPath: string,
  eventType?: string
): {
  events: T[];
  latest: T | null;
  status: 'connecting' | 'streaming' | 'error';
} {
  const [events, setEvents] = useState<T[]>([]);
  const [status, setStatus] = useState<'connecting' | 'streaming' | 'error'>('connecting');

  useEffect(() => {
    const wasm = getWasm();
    const handle = wasm.subscribe_lex_stream(lexPath, eventType ?? '*', (payload: Uint8Array) => {
      const event = parseWasmResponse<T>(payload);
      setEvents((prev) => [...prev.slice(-99), event]);
      setStatus('streaming');
    });
    setStatus('connecting');
    return () => {
      wasm.unsubscribe_lex_stream(handle);
    };
  }, [lexPath, eventType]);

  return { events, latest: events[events.length - 1] ?? null, status };
}

/**
 * Get render-ready widget data from WASM.
 * WASM processes stream data + event bus events -> returns JSON for rendering.
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
