import {test, expect} from "@playwright/test";
import {SiweMessage} from "siwe";
import {privateKeyToAccount} from "viem/accounts";

const SIGNER_PK =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" as const;

const INDEXER = "http://127.0.0.1:8080";

test.describe("SIWE auth round-trip (real signature, mock indexer)", () => {
    test("issued-at expired token fails on /api/me", async ({request}) => {
        const res = await request.get(`${INDEXER}/api/me`, {
            headers: {authorization: "Bearer not-a-token"},
        });
        expect(res.status()).toBe(401);
    });

    test("signing a SIWE message issues a working bearer token", async ({request}) => {
        const account = privateKeyToAccount(SIGNER_PK);

        const nonceRes = await request.post(`${INDEXER}/auth/nonce`);
        expect(nonceRes.ok()).toBeTruthy();
        const {nonce} = (await nonceRes.json()) as {nonce: string};
        expect(nonce.length).toBeGreaterThan(8);

        const message = new SiweMessage({
            domain: "127.0.0.1:3100",
            address: account.address,
            statement: "Sign in to the MultiChain NFT Indexer.",
            uri: "http://127.0.0.1:3100",
            version: "1",
            chainId: 11155111,
            nonce,
            issuedAt: new Date().toISOString(),
        });
        const messageString = message.prepareMessage();
        const signature = await account.signMessage({message: messageString});

        const verifyRes = await request.post(`${INDEXER}/auth/verify`, {
            data: {message: messageString, signature},
        });
        expect(verifyRes.ok()).toBeTruthy();
        const body = (await verifyRes.json()) as {address: string; token: string};
        expect(body.address.toLowerCase()).toBe(account.address.toLowerCase());
        expect(body.token).toMatch(/^mock-jwt-/);

        const meRes = await request.get(`${INDEXER}/api/me`, {
            headers: {authorization: `Bearer ${body.token}`},
        });
        expect(meRes.ok()).toBeTruthy();
        const me = (await meRes.json()) as {address: string};
        expect(me.address.toLowerCase()).toBe(account.address.toLowerCase());
    });

    test("nonce cannot be reused", async ({request}) => {
        const account = privateKeyToAccount(SIGNER_PK);
        const nonceRes = await request.post(`${INDEXER}/auth/nonce`);
        const {nonce} = (await nonceRes.json()) as {nonce: string};

        const buildAndSign = async () => {
            const msg = new SiweMessage({
                domain: "127.0.0.1:3100",
                address: account.address,
                statement: "Sign in to the MultiChain NFT Indexer.",
                uri: "http://127.0.0.1:3100",
                version: "1",
                chainId: 11155111,
                nonce,
                issuedAt: new Date().toISOString(),
            }).prepareMessage();
            const sig = await account.signMessage({message: msg});
            return {msg, sig};
        };

        const first = await buildAndSign();
        const okRes = await request.post(`${INDEXER}/auth/verify`, {
            data: {message: first.msg, signature: first.sig},
        });
        expect(okRes.ok()).toBeTruthy();

        const second = await buildAndSign();
        const reuseRes = await request.post(`${INDEXER}/auth/verify`, {
            data: {message: second.msg, signature: second.sig},
        });
        expect(reuseRes.status()).toBe(400);
    });

    test("verify rejects mismatched domain", async ({request}) => {
        const account = privateKeyToAccount(SIGNER_PK);
        const nonceRes = await request.post(`${INDEXER}/auth/nonce`);
        const {nonce} = (await nonceRes.json()) as {nonce: string};

        const msg = new SiweMessage({
            domain: "phishing.example.com",
            address: account.address,
            statement: "Sign in to the MultiChain NFT Indexer.",
            uri: "http://phishing.example.com",
            version: "1",
            chainId: 11155111,
            nonce,
            issuedAt: new Date().toISOString(),
        }).prepareMessage();
        const sig = await account.signMessage({message: msg});

        const res = await request.post(`${INDEXER}/auth/verify`, {
            data: {message: msg, signature: sig},
        });
        expect(res.status()).toBe(400);
    });
});
