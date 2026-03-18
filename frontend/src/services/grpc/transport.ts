import { createGrpcWebTransport } from "@connectrpc/connect-web";

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;
if (!API_BASE_URL) {
  throw new Error("VITE_API_BASE_URL must be set");
}

/** gRPC-web transport for all service clients. */
export const transport = createGrpcWebTransport({
  baseUrl: API_BASE_URL,
});
