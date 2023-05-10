import * as dotenv from "dotenv";
dotenv.config({
  path: "../../.env",
});
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
console.log(process.env.APP_API_URL);

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  base: process.env.BASE_URL ?? "/",
  build: {
    sourcemap: true,
  },
  mode: "development",
  envPrefix: "APP_",
  envDir: "../../",

  server: {
    proxy: {
      "/api": {
        target: process.env.APP_API_URL as string,
        changeOrigin: true,
      },
    },
  },
});
