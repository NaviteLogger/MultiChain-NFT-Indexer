import EventFeed from "@/components/EventFeed";
import SiweLogin from "@/components/SiweLogin";
import StatsPanel from "@/components/StatsPanel";

export default function Home() {
    return (
        <main className="min-h-screen px-6 py-10 max-w-5xl mx-auto space-y-6">
            <header>
                <h1 className="text-3xl font-semibold tracking-tight">
                    MultiChain NFT Indexer
                </h1>
                <p className="mt-2 text-zinc-400">
                    Rust + Postgres indexer for ERC-721 events. Connect a wallet, sign in
                    with Ethereum, watch events stream live.
                </p>
            </header>

            <StatsPanel />
            <SiweLogin />
            <EventFeed />

            <footer className="text-xs text-zinc-500">
                <p>
                    REST + WebSocket spec:{" "}
                    <span className="mono">indexer/openapi.yaml</span>
                </p>
            </footer>
        </main>
    );
}
