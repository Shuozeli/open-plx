import { createClient } from "@connectrpc/connect";
import { DashboardService } from "../../gen/open_plx/v1/dashboard_pb.js";
import { DataSourceService } from "../../gen/open_plx/v1/data_source_pb.js";
import { transport } from "./transport.js";

/** DashboardService gRPC client. */
export const dashboardClient = createClient(DashboardService, transport);

/** DataSourceService gRPC client. */
export const dataSourceClient = createClient(DataSourceService, transport);
