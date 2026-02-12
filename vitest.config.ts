import path from "path";
import { defineConfig } from "vitest/config";
import { nodePolyfills } from "vite-plugin-node-polyfills";

export default defineConfig({
  plugins: [
    nodePolyfills({
      include: ["buffer", "process", "util", "os", "crypto", "stream"],
      globals: {
        Buffer: true,
        process: true,
        global: true,
      },
    }),
  ],
  resolve: {
    alias: {
      buffer: "buffer",
      process: "process/browser",
      util: "util",
      os: "os-browserify/browser",
      "@alphahuman/skill-types": path.resolve(
        __dirname,
        "src/lib/skills/types.ts"
      ),
    },
  },
  test: {
    globals: true,
    environment: "jsdom",
    mockReset: true,
    restoreMocks: true,
    setupFiles: ["src/test/setup.ts"],
    include: ["src/**/*.test.{ts,tsx}"],
    hookTimeout: 30000,
    testTimeout: 30000,
    coverage: {
      provider: "v8",
      include: ["src/**/*.{ts,tsx}"],
      exclude: [
        "src/main.tsx",
        "src/vite-env.d.ts",
        "src/**/*.d.ts",
        "src/test/**",
        "src/__tests__/**",
        "src/**/__tests__/**",
        "src/**/*.test.{ts,tsx}",
        "src/**/types.ts",
        "src/**/types/*.ts",
        "src/types/**",
      ],
      reporter: ["text", "text-summary", "html", "lcov"],
      thresholds: {
        lines: 90,
        statements: 90,
        functions: 90,
        branches: 85,
      },
    },
  },
});
