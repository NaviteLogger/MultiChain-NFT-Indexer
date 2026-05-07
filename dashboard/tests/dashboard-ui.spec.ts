import {test, expect} from "@playwright/test";

test.describe("Dashboard UI", () => {
    test("page renders all three panels", async ({page}) => {
        await page.goto("/");
        await expect(page.getByRole("heading", {name: /MultiChain NFT Indexer/})).toBeVisible();
        await expect(page.getByTestId("stats-panel")).toBeVisible();
        await expect(page.getByTestId("siwe-panel")).toBeVisible();
        await expect(page.getByTestId("feed-panel")).toBeVisible();
    });

    test("stats panel shows numbers from the indexer", async ({page}) => {
        await page.goto("/");
        const stats = page.getByTestId("stats-panel");
        await expect(stats).toBeVisible();
        await expect(stats).toContainText("12");
        await expect(stats).toContainText("7");
        await expect(stats).toContainText("5");
    });

    test("event feed lists Transfer + Minted rows", async ({page}) => {
        await page.goto("/");
        const feed = page.getByTestId("feed-panel");
        await expect(feed).toContainText("Transfer");
        await expect(feed).toContainText("Minted");
        await expect(feed).toContainText("ipfs://Qm/4.json");
    });

    test("Connect Wallet button opens RainbowKit modal", async ({page}) => {
        await page.goto("/");
        const connect = page
            .getByTestId("siwe-panel")
            .getByRole("button", {name: /Connect Wallet/i});
        await expect(connect).toBeVisible();
        await connect.click();
        const modalTitle = page.getByRole("heading", {name: /Connect a Wallet/i});
        await expect(modalTitle).toBeVisible({timeout: 10_000});
        await page.keyboard.press("Escape");
        await expect(modalTitle).toBeHidden();
    });
});
