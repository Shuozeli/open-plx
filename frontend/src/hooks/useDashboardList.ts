import { useCallback, useEffect, useState } from "react";
import type { Dashboard } from "../gen/open_plx/v1/dashboard_pb.js";
import { dashboardClient } from "../services/grpc/client.js";

interface UseDashboardListResult {
  dashboards: Dashboard[];
  loading: boolean;
  error: string | null;
  refresh: () => void;
}

export function useDashboardList(): UseDashboardListResult {
  const [dashboards, setDashboards] = useState<Dashboard[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await dashboardClient.listDashboards({});
      setDashboards(result.dashboards);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  return { dashboards, loading, error, refresh: load };
}
