import { ConfigProvider, Layout, Typography } from "antd";
import { useEffect, useState } from "react";
import { DashboardPage } from "./pages/DashboardPage.js";

const { Header, Content } = Layout;

function useDashboardName(): string {
  const [name, setName] = useState(() => {
    const hash = window.location.hash.slice(1);
    return hash || "dashboards/demo";
  });

  useEffect(() => {
    const handler = () => setName(window.location.hash.slice(1) || "dashboards/demo");
    window.addEventListener("hashchange", handler);
    return () => window.removeEventListener("hashchange", handler);
  }, []);

  return name;
}

export function App() {
  const dashboardName = useDashboardName();

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
