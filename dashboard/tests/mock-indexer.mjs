import http from "node:http";
import {randomUUID} from "node:crypto";
import {SiweMessage} from "siwe";

const PORT = Number(process.env.MOCK_INDEXER_PORT ?? 8080);
const DOMAIN = process.env.MOCK_INDEXER_SIWE_DOMAIN ?? "localhost:3000";

const nonces = new Map();
const tokens = new Map();

const STATS = {
    events_total: 12,
    nfts_total: 7,
    holders_total: 5,
    last_block: 6_500_321,
};

const EVENTS = Array.from({length: 5}, (_, i) => ({
    id: 100 - i,
    chain_id: 11155111,
    contract: "0x0000000000000000000000000000000000000123",
    event: i === 4 ? "Minted" : "Transfer",
    from_address: i === 4 ? null : "0x0000000000000000000000000000000000000000",
    to_address: "0xabc0000000000000000000000000000000000001",
    token_id: String(i),
    uri: i === 4 ? `ipfs://Qm/${i}.json` : null,
    tx_hash: "0x" + "ab".repeat(32),
    log_index: i,
    block_number: 6_500_300 - i,
    block_timestamp: new Date(Date.now() - i * 60_000).toISOString(),
}));

function send(res, status, body, extraHeaders = {}) {
    const payload =
        typeof body === "string" || Buffer.isBuffer(body) ? body : JSON.stringify(body);
    res.writeHead(status, {
        "content-type": typeof body === "string" ? "text/plain" : "application/json",
        "access-control-allow-origin": "*",
        "access-control-allow-headers": "authorization, content-type",
        "access-control-allow-methods": "GET, POST, OPTIONS",
        ...extraHeaders,
    });
    res.end(payload);
}

async function readJson(req) {
    return new Promise((resolve, reject) => {
        const chunks = [];
        req.on("data", (c) => chunks.push(c));
        req.on("end", () => {
            try {
                const buf = Buffer.concat(chunks).toString("utf8");
                resolve(buf ? JSON.parse(buf) : {});
            } catch (err) {
                reject(err);
            }
        });
        req.on("error", reject);
    });
}

const server = http.createServer(async (req, res) => {
    if (req.method === "OPTIONS") return send(res, 204, "");

    const url = new URL(req.url, "http://x");
    const path = url.pathname;

    try {
        if (req.method === "GET" && path === "/healthz") return send(res, 200, "ok");

        if (req.method === "POST" && path === "/auth/nonce") {
            const nonce = randomUUID().replaceAll("-", "");
            nonces.set(nonce, false);
            return send(res, 200, {nonce});
        }

        if (req.method === "POST" && path === "/auth/verify") {
            const body = await readJson(req);
            if (typeof body?.message !== "string" || typeof body?.signature !== "string") {
                return send(res, 400, {error: "missing fields"});
            }
            let msg;
            try {
                msg = new SiweMessage(body.message);
            } catch (err) {
                return send(res, 400, {error: `bad SIWE: ${err.message}`});
            }
            if (msg.domain !== DOMAIN) {
                return send(res, 400, {error: "domain mismatch"});
            }
            if (!nonces.has(msg.nonce)) {
                return send(res, 400, {error: "unknown nonce"});
            }
            if (nonces.get(msg.nonce)) {
                return send(res, 400, {error: "nonce already used"});
            }
            try {
                await msg.verify({signature: body.signature, nonce: msg.nonce});
            } catch (err) {
                return send(res, 400, {error: `verify: ${err.error?.type ?? err.message}`});
            }
            nonces.set(msg.nonce, true);
            const address = msg.address.toLowerCase();
            const token = `mock-jwt-${randomUUID()}`;
            tokens.set(token, address);
            return send(res, 200, {address, token});
        }

        if (req.method === "GET" && path === "/api/me") {
            const auth = req.headers["authorization"];
            const token = typeof auth === "string" ? auth.replace(/^Bearer\s+/, "") : null;
            const address = token ? tokens.get(token) : null;
            if (!address) return send(res, 401, {error: "unauthorized"});
            return send(res, 200, {address});
        }

        if (req.method === "GET" && path === "/api/stats") return send(res, 200, STATS);

        if (req.method === "GET" && path === "/api/events") return send(res, 200, EVENTS);

        if (req.method === "GET" && path.startsWith("/api/holders/"))
            return send(res, 200, []);

        if (req.method === "GET" && path.startsWith("/api/nfts/similar/")) {
            const parts = path.split("/").filter(Boolean);
            // /api/nfts/similar/{contract}/{token_id}
            if (parts.length < 5)
                return send(res, 400, {error: "invalid similar path"});
            const contract = parts[3];
            const tokenId = parts[4];
            return send(res, 200, [
                {
                    chain_id: 11155111,
                    contract,
                    token_id: String(Number(tokenId) + 1),
                    current_owner: "0xabc0000000000000000000000000000000000002",
                    uri: "ipfs://Qm/neighbour-1.json",
                    distance: 0.04,
                },
                {
                    chain_id: 11155111,
                    contract,
                    token_id: String(Number(tokenId) + 7),
                    current_owner: "0xabc0000000000000000000000000000000000003",
                    uri: "ipfs://Qm/neighbour-2.json",
                    distance: 0.18,
                },
            ]);
        }

        if (req.method === "GET" && path.startsWith("/api/nfts/"))
            return send(res, 404, {error: "not found"});

        return send(res, 404, {error: `no route for ${req.method} ${path}`});
    } catch (err) {
        return send(res, 500, {error: String(err)});
    }
});

server.listen(PORT, () => {
    console.log(`mock indexer listening on :${PORT}`);
});
