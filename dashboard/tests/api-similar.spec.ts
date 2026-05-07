import {test, expect} from "@playwright/test";

const INDEXER = "http://127.0.0.1:8080";
const CONTRACT = "0x0000000000000000000000000000000000000123";

test.describe("/api/nfts/similar/{contract}/{token_id}", () => {
    test("returns nearest neighbours sorted by ascending distance", async ({request}) => {
        const res = await request.get(`${INDEXER}/api/nfts/similar/${CONTRACT}/0`);
        expect(res.ok()).toBeTruthy();
        const body = (await res.json()) as Array<{
            contract: string;
            token_id: string;
            distance: number;
        }>;
        expect(body.length).toBeGreaterThan(0);
        expect(body[0].contract.toLowerCase()).toBe(CONTRACT);
        for (let i = 1; i < body.length; i++) {
            expect(body[i].distance).toBeGreaterThanOrEqual(body[i - 1].distance);
        }
    });

    test("includes a uri and distance for every neighbour", async ({request}) => {
        const res = await request.get(`${INDEXER}/api/nfts/similar/${CONTRACT}/0?limit=2`);
        const body = await res.json();
        for (const item of body) {
            expect(typeof item.distance).toBe("number");
            expect(item.distance).toBeGreaterThanOrEqual(0);
            expect(item.uri).toMatch(/^ipfs:\/\//);
        }
    });
});
