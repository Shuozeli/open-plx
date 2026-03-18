import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  server: {
    host: "0.0.0.0",
    proxy: {
      // Proxy gRPC-web requests to the backend
      "/open_plx.v1": {
        target: "http://localhost:50051",
        changeOrigin: true,
      },
      "/grpc.reflection": {
        target: "http://localhost:50051",
        changeOrigin: true,
      },
    },
  },
});
