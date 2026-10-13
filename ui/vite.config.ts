import adapter from "@sveltejs/adapter-static";
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

export default defineConfig(({ mode }) => {
  const isTest = mode === "test";

  return {
    plugins: [
      sveltekit({
        adapter: adapter({
          pages: "build",
          assets: "build",
          fallback: "index.html",
        }),
      }),
    ],
    resolve: isTest ? { conditions: ["browser"] } : undefined,
    test: {
      environment: "jsdom",
    },
    server: {
      proxy: {
        "/api": "http://localhost:4110",
        "/auth": "http://localhost:4110",
        "/health": "http://localhost:4110",
      },
    },
  };
});
