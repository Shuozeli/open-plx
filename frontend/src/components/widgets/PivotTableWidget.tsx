import { Card, Spin } from "antd";
import { useEffect, useRef, useState } from "react";
import type { WidgetProps } from "./WidgetRegistry.js";
import { pivotProtoToS2, pivotProtoToTestState } from "../../services/mappers/pivotMapper.js";
import { S2PivotTable } from "./S2PivotTable.js";
import { registerWidget } from "../../services/testRegistry.js";
import { WidgetType } from "../../gen/open_plx/v1/dashboard_pb.js";

export function PivotTableWidget({ config, data, loading, error }: WidgetProps) {
  const spec = config.spec?.spec.case === "pivotTable" ? config.spec.spec.value : null;
  const bodyRef = useRef<HTMLDivElement>(null);
  const [dims, setDims] = useState({ w: 800, h: 300 });

  // Measure the card body for S2 width/height
  useEffect(() => {
    if (!bodyRef.current) return;
    const ro = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setDims({
          w: Math.floor(entry.contentRect.width),
          h: Math.floor(entry.contentRect.height),
        });
      }
    });
    ro.observe(bodyRef.current);
    return () => ro.disconnect();
  }, []);

  // Register test state
  useEffect(() => {
    registerWidget({
      widgetId: config.id,
      widgetType: WidgetType[config.widgetType],
      spec: spec ? JSON.parse(JSON.stringify(spec)) : {},
      data,
      g2Spec: null,
      rendered: {
        hasData: data !== null && data.length > 0,
        rowCount: data?.length ?? 0,
        ...(spec && data ? pivotProtoToTestState(spec, data) : {}),
      },
      updatedAt: Date.now(),
    });
  }, [config, data, spec]);

  if (error) {
    return <Card title={config.title}><span>Error: {error}</span></Card>;
  }

  if (loading || !data || !spec) {
    return <Card title={config.title} style={{ height: "100%" }}><Spin /></Card>;
  }

  const { dataCfg, options } = pivotProtoToS2(spec, data, dims.w, dims.h);

  return (
    <Card
      title={config.title}
      style={{ height: "100%" }}
      styles={{ body: { height: "calc(100% - 56px)", padding: 0, overflow: "hidden" } }}
    >
      <div ref={bodyRef} style={{ width: "100%", height: "100%" }}>
        {dims.w > 0 && dims.h > 0 && (
          <S2PivotTable dataCfg={dataCfg} options={options} />
        )}
      </div>
    </Card>
  );
}
