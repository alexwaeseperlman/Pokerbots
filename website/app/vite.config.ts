import * as dotenv from "dotenv";
dotenv.config({
  path: "../../.env",
});
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  base: process.env.BASE_URL ?? "/",
  build: {
    sourcemap: true,
  },
  envPrefix: "APP_",
  envDir: "../../",
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:3000",
        changeOrigin: true,
        secure: false,
        ws: true,
      },
    },
  },
});
