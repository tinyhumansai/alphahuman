#!/usr/bin/env node
import process from "node:process";

import { DEFAULT_PORT, startMockServer, stopMockServer } from "./mock-api-core.mjs";

function readPortArg() {
  const idx = process.argv.findIndex((arg) => arg === "--port" || arg === "-p");
  if (idx >= 0 && process.argv[idx + 1]) {
    const value = Number(process.argv[idx + 1]);
    if (Number.isInteger(value) && value > 0) {
      return value;
    }
  }
  const envPort = Number(process.env.MOCK_API_PORT || process.env.E2E_MOCK_PORT || DEFAULT_PORT);
  return Number.isInteger(envPort) && envPort > 0 ? envPort : DEFAULT_PORT;
}

async function main() {
  const port = readPortArg();
  await startMockServer(port);
  const shutdown = async () => {
    await stopMockServer();
    process.exit(0);
  };
  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);
}

main().catch((err) => {
  console.error("[mock-api-server] Failed to start:", err);
  process.exit(1);
});
