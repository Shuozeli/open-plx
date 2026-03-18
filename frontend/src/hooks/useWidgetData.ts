import { useCallback, useEffect, useState } from "react";

// TODO(refactor): Replace with Arrow Flight client when data service is implemented.
// For now, widget data is not fetched -- widgets render with spec only.

interface UseWidgetDataResult {
  data: Record<string, unknown>[] | null;
  loading: boolean;
  error: string | null;
  refresh: () => void;
}

/** Placeholder hook for widget data fetching (Arrow Flight). */
export function useWidgetData(
  _dashboardName: string,
  _widgetId: string,
): UseWidgetDataResult {
  const [data] = useState<Record<string, unknown>[] | null>(null);
  const [loading] = useState(false);
  const [error] = useState<string | null>(null);

  const refresh = useCallback(() => {
    // TODO(refactor): Implement Arrow Flight data fetch.
  }, []);

  useEffect(() => {
    // No-op until Arrow Flight is wired up.
  }, []);

  return { data, loading, error, refresh };
}
