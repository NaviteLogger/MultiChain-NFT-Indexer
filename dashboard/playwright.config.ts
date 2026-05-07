import {defineConfig, devices} from "@playwright/test";

const dashboardURL = "http://127.0.0.1:3100";

export default defineConfig({
    testDir: "./tests",
    fullyParallel: true,
    forbidOnly: !!process.env.CI,
    retries: process.env.CI ? 1 : 0,
    workers: 1,
    reporter: process.env.CI ? "github" : "list",
    use: {
        baseURL: dashboardURL,
        trace: "retain-on-failure",
        screenshot: "only-on-failure",
    },
    projects: [
        {
            name: "chromium",
            use: {...devices["Desktop Chrome"]},
        },
    ],
    webServer: [
        {
            command: "node tests/mock-indexer.mjs",
            url: "http://127.0.0.1:8080/healthz",
            reuseExistingServer: !process.env.CI,
            stdout: "pipe",
            stderr: "pipe",
            timeout: 30_000,
            env: {
                MOCK_INDEXER_PORT: "8080",
                MOCK_INDEXER_SIWE_DOMAIN: "127.0.0.1:3100",
            },
        },
        {
            command: "npm run dev -- -p 3100",
            url: dashboardURL,
            reuseExistingServer: !process.env.CI,
            stdout: "pipe",
            stderr: "pipe",
            timeout: 120_000,
            env: {
                NEXT_PUBLIC_INDEXER_URL: "http://127.0.0.1:8080",
                NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID: "demo",
                NEXT_PUBLIC_DEFAULT_CHAIN_ID: "11155111",
            },
        },
    ],
});
