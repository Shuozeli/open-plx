import { Component } from "react";
import type { ErrorInfo, ReactNode } from "react";
import { Card, Result } from "antd";
import { WarningOutlined } from "@ant-design/icons";

interface Props {
  title: string;
  children: ReactNode;
}

interface State {
  hasError: boolean;
  errorMessage: string;
}

/**
 * Error boundary that catches rendering errors within a single widget.
 * Prevents one broken widget from crashing the entire dashboard.
 */
export class WidgetErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, errorMessage: "" };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, errorMessage: error.message };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error(`Widget "${this.props.title}" crashed:`, error, info.componentStack);
  }

  render() {
    if (this.state.hasError) {
      return (
        <Card title={this.props.title} style={{ height: "100%" }}>
          <Result
            icon={<WarningOutlined />}
            title="Widget Error"
            subTitle={this.state.errorMessage || "An unexpected error occurred."}
            status="error"
          />
        </Card>
      );
    }

    return this.props.children;
  }
}
