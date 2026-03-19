import { ConfigProvider, Layout, Switch, Typography, theme } from "antd";
import { BulbOutlined } from "@ant-design/icons";
import { useEffect, useState } from "react";
import { DashboardPage } from "./pages/DashboardPage.js";
import { DashboardListPage } from "./pages/DashboardListPage.js";
import { ThemeContext } from "./hooks/useThemeContext.js";

const { Header, Content } = Layout;

function useHash(): string {
  const [hash, setHash] = useState(() => window.location.hash.slice(1));

  useEffect(() => {
    const handler = () => setHash(window.location.hash.slice(1));
    window.addEventListener("hashchange", handler);
    return () => window.removeEventListener("hashchange", handler);
  }, []);

  return hash;
}

function useTheme(): [boolean, () => void] {
  const [dark, setDark] = useState(() => {
    const stored = localStorage.getItem("open-plx-theme");
    if (stored === "dark") return true;
    if (stored === "light") return false;
    return window.matchMedia("(prefers-color-scheme: dark)").matches;
  });

  const toggle = () => {
    setDark((prev) => {
      const next = !prev;
      localStorage.setItem("open-plx-theme", next ? "dark" : "light");
      return next;
    });
  };

  return [dark, toggle];
}

export function App() {
  const hash = useHash();
  const isDashboard = hash.startsWith("dashboards/");
  const [dark, toggleTheme] = useTheme();

  return (
    <ThemeContext.Provider value={{ dark }}>
    <ConfigProvider
      theme={{
        algorithm: dark ? theme.darkAlgorithm : theme.defaultAlgorithm,
      }}
    >
      <Layout style={{ minHeight: "100vh" }}>
        <Header style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <a href="#" style={{ textDecoration: "none" }}>
            <Typography.Title level={4} style={{ color: "#fff", margin: 0 }}>
              open-plx
            </Typography.Title>
          </a>
          <Switch
            checked={dark}
            onChange={toggleTheme}
            checkedChildren={<BulbOutlined />}
            unCheckedChildren={<BulbOutlined />}
          />
        </Header>
        <Content style={{ padding: 24, background: dark ? "#141414" : "#f5f5f5" }}>
          {isDashboard ? (
            <DashboardPage dashboardName={hash} />
          ) : (
            <DashboardListPage />
          )}
        </Content>
      </Layout>
    </ConfigProvider>
    </ThemeContext.Provider>
  );
}
