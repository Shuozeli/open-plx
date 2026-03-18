import { ConfigProvider, Layout, Typography } from "antd";
import { DashboardPage } from "./pages/DashboardPage.js";

const { Header, Content } = Layout;

export function App() {
  // TODO(refactor): Add routing for dashboard list and individual dashboards.
  const dashboardName = "dashboards/demo";

  return (
    <ConfigProvider>
      <Layout style={{ minHeight: "100vh" }}>
        <Header style={{ display: "flex", alignItems: "center" }}>
          <Typography.Title level={4} style={{ color: "#fff", margin: 0 }}>
            open-plx
          </Typography.Title>
        </Header>
        <Content style={{ padding: 24, background: "#f5f5f5" }}>
          <DashboardPage dashboardName={dashboardName} />
        </Content>
      </Layout>
    </ConfigProvider>
  );
}
