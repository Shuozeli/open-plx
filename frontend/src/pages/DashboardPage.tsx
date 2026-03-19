import { Alert, Button, Spin, Typography } from "antd";
import { ReloadOutlined } from "@ant-design/icons";
import { useEffect } from "react";
import { useDashboard } from "../hooks/useDashboard.js";
import { useVariables } from "../hooks/useVariables.js";
import { DashboardGrid } from "../components/layout/DashboardGrid.js";
import { VariableBar } from "../components/variables/VariableBar.js";
import { useDarkMode } from "../hooks/useThemeContext.js";
import { initTestRegistry, registerDashboard, registerDashboardStatus, registerTheme } from "../services/testRegistry.js";

interface DashboardPageProps {
  dashboardName: string;
}

export function DashboardPage({ dashboardName }: DashboardPageProps) {
  const { dashboard, loading, error, refresh } = useDashboard(dashboardName);
  const { values: variableValues, setValue: setVariableValue, revision } = useVariables(
    dashboard?.variables ?? [],
  );
  const dark = useDarkMode();

  // Initialize test registry on mount
  useEffect(() => {
    initTestRegistry();
  }, []);

  // Register theme for e2e tests
  useEffect(() => {
    registerTheme(dark);
  }, [dark]);

  // Register dashboard state for e2e tests
  useEffect(() => {
    registerDashboardStatus(loading, error);
    if (dashboard) {
      registerDashboard({
        name: dashboard.name,
        title: dashboard.title,
        widgetCount: dashboard.widgets.length,
        grid: {
          columns: dashboard.grid?.columns ?? 0,
          rowHeight: dashboard.grid?.rowHeight ?? 0,
          gap: dashboard.grid?.gap ?? 0,
        },
      });
    }
  }, [dashboard, loading, error]);

  if (loading) {
    return <Spin size="large" style={{ display: "block", margin: "100px auto" }} />;
  }

  if (error) {
    return <Alert type="error" title="Failed to load dashboard" description={error} />;
  }

  if (!dashboard) {
    return <Alert type="warning" title="Dashboard not found" />;
  }

  return (
    <div style={{ padding: 24 }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
        <div>
          <Typography.Title level={2} style={{ margin: 0 }}>{dashboard.title}</Typography.Title>
          {dashboard.description && (
            <Typography.Text type="secondary">{dashboard.description}</Typography.Text>
          )}
        </div>
        <Button icon={<ReloadOutlined />} onClick={refresh}>Refresh</Button>
      </div>
      <VariableBar
        variables={dashboard.variables}
        values={variableValues}
        onValueChange={setVariableValue}
      />
      <DashboardGrid dashboard={dashboard} variableValues={variableValues} revision={revision} />
    </div>
  );
}
