import { Alert, Button, Spin, Typography } from "antd";
import { ReloadOutlined } from "@ant-design/icons";
import { useDashboard } from "../hooks/useDashboard.js";
import { DashboardGrid } from "../components/layout/DashboardGrid.js";

interface DashboardPageProps {
  dashboardName: string;
}

export function DashboardPage({ dashboardName }: DashboardPageProps) {
  const { dashboard, loading, error, refresh } = useDashboard(dashboardName);

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
      <DashboardGrid dashboard={dashboard} />
    </div>
  );
}
