import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

export default defineConfig(({ mode }) => {
  const isTest = mode === "test";

  return {
    plugins: [sveltekit()],
    resolve: isTest ? { conditions: ["browser"] } : undefined,
    test: {
      environment: "jsdom",
    },
    server: {
      proxy: {
        "/api": "http://localhost:4100",
        "/health": "http://localhost:4100",
      },
    },
  };
});
