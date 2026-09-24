import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Serveur de développement uniquement pour la fenêtre Tauri (aucune exposition réseau).
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true, host: "127.0.0.1" },
  build: { target: "safari16", sourcemap: false },
  test: { include: ["src/**/*.test.ts"] },
});
