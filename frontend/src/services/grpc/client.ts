import { createClient } from "@connectrpc/connect";
import { DashboardService } from "../../gen/open_plx/v1/dashboard_pb.js";
import { WidgetDataService } from "../../gen/open_plx/v1/data_pb.js";
import { transport } from "./transport.js";

export const dashboardClient = createClient(DashboardService, transport);
export const widgetDataClient = createClient(WidgetDataService, transport);
