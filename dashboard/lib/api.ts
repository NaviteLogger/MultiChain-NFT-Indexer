export const INDEXER_URL =
    process.env.NEXT_PUBLIC_INDEXER_URL || "http://localhost:8080";

const TOKEN_KEY = "indexer.jwt";

export function getToken(): string | null {
    if (typeof window === "undefined") return null;
    return window.localStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string | null) {
    if (typeof window === "undefined") return;
    if (token === null) {
        window.localStorage.removeItem(TOKEN_KEY);
    } else {
        window.localStorage.setItem(TOKEN_KEY, token);
    }
}

export interface Stats {
    events_total: number;
    nfts_total: number;
    holders_total: number;
    last_block: number | null;
}

export interface IndexedEvent {
    id: number;
    chain_id: number;
    contract: string;
    event: "Transfer" | "Minted";
    from_address: string | null;
    to_address: string;
    token_id: string;
    uri: string | null;
    tx_hash: string;
    log_index: number;
    block_number: number;
    block_timestamp: string;
}

async function get<T>(path: string, init?: RequestInit): Promise<T> {
    const res = await fetch(`${INDEXER_URL}${path}`, {
        ...init,
        headers: {
            "content-type": "application/json",
            ...(init?.headers ?? {}),
        },
    });
    if (!res.ok) {
        const text = await res.text();
        throw new Error(`${res.status} ${path}: ${text}`);
    }
    return (await res.json()) as T;
}

export async function fetchStats(): Promise<Stats> {
    return get<Stats>("/api/stats");
}

export async function fetchEvents(params: {
    limit?: number;
    chainId?: number;
    contract?: string;
    beforeId?: number;
} = {}): Promise<IndexedEvent[]> {
    const qs = new URLSearchParams();
    if (params.limit) qs.set("limit", String(params.limit));
    if (params.chainId) qs.set("chain_id", String(params.chainId));
    if (params.contract) qs.set("contract", params.contract);
    if (params.beforeId) qs.set("before_id", String(params.beforeId));
    const q = qs.toString();
    return get<IndexedEvent[]>(`/api/events${q ? `?${q}` : ""}`);
}

export async function fetchNonce(): Promise<string> {
    const res = await fetch(`${INDEXER_URL}/auth/nonce`, {method: "POST"});
    if (!res.ok) throw new Error(`nonce: ${res.status}`);
    const body = (await res.json()) as {nonce: string};
    return body.nonce;
}

export async function siweVerify(message: string, signature: string): Promise<{address: string; token: string}> {
    const res = await fetch(`${INDEXER_URL}/auth/verify`, {
        method: "POST",
        headers: {"content-type": "application/json"},
        body: JSON.stringify({message, signature}),
    });
    if (!res.ok) {
        const text = await res.text();
        throw new Error(`verify: ${text}`);
    }
    return (await res.json()) as {address: string; token: string};
}

export async function fetchMe(): Promise<{address: string} | null> {
    const token = getToken();
    if (!token) return null;
    const res = await fetch(`${INDEXER_URL}/api/me`, {
        headers: {authorization: `Bearer ${token}`},
    });
    if (res.status === 401) {
        setToken(null);
        return null;
    }
    if (!res.ok) throw new Error(`me: ${res.status}`);
    return (await res.json()) as {address: string};
}

export function wsUrl(): string {
    const base = INDEXER_URL.replace(/^http/, "ws");
    return `${base}/ws/events`;
}
