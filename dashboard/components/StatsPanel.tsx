"use client";

import {useEffect, useState} from "react";
import {fetchStats, type Stats} from "@/lib/api";

export default function StatsPanel() {
    const [stats, setStats] = useState<Stats | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        let cancelled = false;
        const tick = async () => {
            try {
                const s = await fetchStats();
                if (!cancelled) setStats(s);
            } catch (err) {
                if (!cancelled)
                    setError(err instanceof Error ? err.message : String(err));
            }
        };
        tick();
        const id = setInterval(tick, 5_000);
        return () => {
            cancelled = true;
            clearInterval(id);
        };
    }, []);

    return (
        <section
            data-testid="stats-panel"
            className="rounded-xl border border-zinc-800 bg-zinc-950/80 p-6"
        >
            <h2 className="text-lg font-semibold mb-4">Indexer stats</h2>
            {error && <p className="text-xs text-rose-400">indexer offline: {error}</p>}
            {stats && (
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                    <Stat label="Events" value={stats.events_total} />
                    <Stat label="NFTs" value={stats.nfts_total} />
                    <Stat label="Holders" value={stats.holders_total} />
                    <Stat label="Last block" value={stats.last_block ?? "—"} />
                </div>
            )}
        </section>
    );
}

function Stat({label, value}: {label: string; value: number | string}) {
    return (
        <div>
            <div className="text-xs uppercase tracking-wider text-zinc-500">{label}</div>
            <div className="text-2xl font-semibold mono">{value}</div>
        </div>
    );
}
