import { useCallback, useEffect, useState } from "react";
import type { Dashboard } from "../gen/open_plx/v1/dashboard_pb.js";
import { dashboardClient } from "../services/grpc/client.js";

interface UseDashboardResult {
  dashboard: Dashboard | null;
  loading: boolean;
  error: string | null;
  refresh: () => void;
}

/** Fetch a dashboard layout via gRPC. */
export function useDashboard(name: string): UseDashboardResult {
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await dashboardClient.getDashboard({ name });
      setDashboard(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setLoading(false);
    }
  }, [name]);

  useEffect(() => {
    void load();
  }, [load]);

  return { dashboard, loading, error, refresh: load };
}
