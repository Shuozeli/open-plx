import { Alert, Card, List, Spin, Typography } from "antd";
import { useDashboardList } from "../hooks/useDashboardList.js";

export function DashboardListPage() {
  const { dashboards, loading, error } = useDashboardList();

  if (loading) {
    return <Spin size="large" style={{ display: "block", margin: "100px auto" }} />;
  }

  if (error) {
    return <Alert type="error" title="Failed to load dashboards" description={error} />;
  }

  return (
    <div style={{ padding: 24, maxWidth: 800, margin: "0 auto" }}>
      <Typography.Title level={2}>Dashboards</Typography.Title>
      <List
        grid={{ gutter: 16, column: 2 }}
        dataSource={dashboards}
        renderItem={(dashboard) => (
          <List.Item>
            <a href={`#${dashboard.name}`} style={{ display: "block" }}>
              <Card hoverable title={dashboard.title}>
                <Typography.Text type="secondary">
                  {dashboard.description || "No description"}
                </Typography.Text>
                <div style={{ marginTop: 8 }}>
                  <Typography.Text>
                    {dashboard.widgets.length} widget{dashboard.widgets.length !== 1 ? "s" : ""}
                  </Typography.Text>
                </div>
              </Card>
            </a>
          </List.Item>
        )}
      />
    </div>
  );
}
