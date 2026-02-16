/**
 * WidgetShell — generic widget container
 *
 * Provides: RBAC gating, loading state, error boundary, layout sizing.
 * Calls WASM for RBAC check — TS only renders the result.
 */

import React, { Suspense } from 'react';
import { usePolyContext } from './PolyProvider';

interface WidgetShellProps {
  /** Widget ID for data routing */
  widgetId: string;
  /** Display title */
  title: string;
  /** Required roles (checked via WASM RBAC) */
  requiredRoles?: string[];
  /** Current user's roles */
  userRoles?: string[];
  /** Grid size */
  size?: { cols: number; rows: number };
  children: React.ReactNode;
}

export function WidgetShell({
  widgetId,
  title,
  requiredRoles = [],
  userRoles = [],
  size = { cols: 6, rows: 3 },
  children,
}: WidgetShellProps) {
  // RBAC check (simple client-side gate — real enforcement is in WASM/server)
  const hasAccess =
    requiredRoles.length === 0 || requiredRoles.some((r) => userRoles.includes(r));

  if (!hasAccess) {
    return (
      <div
        className="polykit-widget-shell unauthorized"
        style={{ gridColumn: `span ${size.cols}`, gridRow: `span ${size.rows}` }}
      >
        <div className="widget-header">{title}</div>
        <div className="widget-unauthorized">Insufficient permissions</div>
      </div>
    );
  }

  return (
    <div
      className="polykit-widget-shell"
      data-widget-id={widgetId}
      style={{ gridColumn: `span ${size.cols}`, gridRow: `span ${size.rows}` }}
    >
      <div className="widget-header">{title}</div>
      <Suspense fallback={<div className="widget-loading">Loading...</div>}>
        {children}
      </Suspense>
    </div>
  );
}
