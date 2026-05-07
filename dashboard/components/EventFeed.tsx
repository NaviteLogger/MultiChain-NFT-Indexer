"use client";

import {useEffect, useRef, useState} from "react";
import {fetchEvents, type IndexedEvent, wsUrl} from "@/lib/api";

const MAX_FEED = 100;

export default function EventFeed() {
    const [events, setEvents] = useState<IndexedEvent[]>([]);
    const [error, setError] = useState<string | null>(null);
    const [wsState, setWsState] = useState<"closed" | "open" | "connecting">(
        "connecting",
    );
    const wsRef = useRef<WebSocket | null>(null);

    useEffect(() => {
        fetchEvents({limit: 50})
            .then(setEvents)
            .catch((err) => setError(err instanceof Error ? err.message : String(err)));
    }, []);

    useEffect(() => {
        let cancelled = false;
        const connect = () => {
            if (cancelled) return;
            try {
                const sock = new WebSocket(wsUrl());
                wsRef.current = sock;
                setWsState("connecting");
                sock.onopen = () => setWsState("open");
                sock.onclose = () => {
                    setWsState("closed");
                    setTimeout(connect, 2_000);
                };
                sock.onerror = () => sock.close();
                sock.onmessage = (ev) => {
                    try {
                        const env = JSON.parse(ev.data) as Omit<IndexedEvent, "id"> & {
                            id?: number;
                        };
                        setEvents((prev) =>
                            [
                                {...env, id: env.id ?? -prev.length - 1},
                                ...prev,
                            ].slice(0, MAX_FEED),
                        );
                    } catch {
                        // swallow malformed messages
                    }
                };
            } catch {
                setWsState("closed");
                setTimeout(connect, 2_000);
            }
        };
        connect();
        return () => {
            cancelled = true;
            wsRef.current?.close();
        };
    }, []);

    return (
        <section
            data-testid="feed-panel"
            className="rounded-xl border border-zinc-800 bg-zinc-950/80 p-6 space-y-3"
        >
            <header className="flex items-center justify-between">
                <h2 className="text-lg font-semibold">Live event feed</h2>
                <span
                    data-testid="ws-state"
                    className={
                        wsState === "open"
                            ? "text-xs text-emerald-400"
                            : wsState === "connecting"
                              ? "text-xs text-zinc-400"
                              : "text-xs text-rose-400"
                    }
                >
                    ws · {wsState}
                </span>
            </header>

            {error && <p className="text-xs text-rose-400">{error}</p>}

            {!events.length && !error && (
                <p className="text-sm text-zinc-400">No events yet.</p>
            )}

            <ul className="divide-y divide-zinc-800">
                {events.map((e, i) => (
                    <li key={`${e.tx_hash}-${e.log_index}-${i}`} className="py-3 text-sm">
                        <div className="flex items-baseline gap-3">
                            <span
                                className={
                                    e.event === "Minted"
                                        ? "text-emerald-400 mono text-xs"
                                        : "text-cyan-400 mono text-xs"
                                }
                            >
                                {e.event}
                            </span>
                            <span className="mono text-xs">#{e.token_id}</span>
                            <span className="text-zinc-500 text-xs">
                                blk {e.block_number}
                            </span>
                        </div>
                        <div className="mono text-xs text-zinc-400 break-all">
                            {e.from_address ?? "0x0000…"} → {e.to_address}
                        </div>
                        {e.uri && (
                            <div className="mono text-xs text-zinc-500 break-all">
                                {e.uri}
                            </div>
                        )}
                    </li>
                ))}
            </ul>
        </section>
    );
}
