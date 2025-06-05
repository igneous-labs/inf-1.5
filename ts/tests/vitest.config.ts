import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    dir: "./test",
    testTimeout: 60000,
    hookTimeout: 30000,
  },
});
