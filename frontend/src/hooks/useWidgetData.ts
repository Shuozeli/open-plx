import { useCallback, useEffect, useState } from "react";
import type { WidgetDataResponse } from "../gen/open_plx/v1/data_pb.js";
import { widgetDataClient } from "../services/grpc/client.js";

interface UseWidgetDataResult {
  data: Record<string, unknown>[] | null;
  rawResponse: WidgetDataResponse | null;
  loading: boolean;
  error: string | null;
  refresh: () => void;
}

/** Convert columnar proto response to row objects for chart libraries. */
function columnsToRows(response: WidgetDataResponse): Record<string, unknown>[] {
  const numRows = Number(response.totalRows);
  if (numRows === 0) return [];

  const rows: Record<string, unknown>[] = new Array(numRows);
  for (let i = 0; i < numRows; i++) {
    rows[i] = {};
  }

  for (const col of response.columns) {
    // Determine which value array is populated
    if (col.stringValues.length > 0) {
      for (let i = 0; i < numRows; i++) {
        rows[i][col.name] = col.stringValues[i];
      }
    } else if (col.doubleValues.length > 0) {
      for (let i = 0; i < numRows; i++) {
        rows[i][col.name] = col.doubleValues[i];
      }
    } else if (col.intValues.length > 0) {
      for (let i = 0; i < numRows; i++) {
        rows[i][col.name] = Number(col.intValues[i]);
      }
    } else if (col.boolValues.length > 0) {
      for (let i = 0; i < numRows; i++) {
        rows[i][col.name] = col.boolValues[i];
      }
    }
  }

  return rows;
}

/** Fetch widget data via WidgetDataService gRPC. */
export function useWidgetData(
  dashboardName: string,
  widgetId: string,
): UseWidgetDataResult {
  const [data, setData] = useState<Record<string, unknown>[] | null>(null);
  const [rawResponse, setRawResponse] = useState<WidgetDataResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await widgetDataClient.getWidgetData({
        dashboard: dashboardName,
        widgetId,
      });
      setRawResponse(response);
      setData(columnsToRows(response));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setLoading(false);
    }
  }, [dashboardName, widgetId]);

  useEffect(() => {
    void load();
  }, [load]);

  return { data, rawResponse, loading, error, refresh: load };
}
