"use client";

import {ConnectButton} from "@rainbow-me/rainbowkit";
import {useEffect, useState} from "react";
import {useAccount, useChainId, useSignMessage} from "wagmi";
import {SiweMessage} from "siwe";
import {fetchMe, fetchNonce, getToken, setToken, siweVerify} from "@/lib/api";

export default function SiweLogin() {
    const {address, isConnected} = useAccount();
    const chainId = useChainId();
    const {signMessageAsync} = useSignMessage();

    const [authedAddress, setAuthedAddress] = useState<string | null>(null);
    const [busy, setBusy] = useState(false);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        if (!getToken()) return;
        fetchMe()
            .then((res) => setAuthedAddress(res?.address ?? null))
            .catch(() => setAuthedAddress(null));
    }, []);

    const signIn = async () => {
        if (!address) return;
        setBusy(true);
        setError(null);
        try {
            const nonce = await fetchNonce();
            const message = new SiweMessage({
                domain: window.location.host,
                address,
                statement: "Sign in to the MultiChain NFT Indexer.",
                uri: window.location.origin,
                version: "1",
                chainId,
                nonce,
                issuedAt: new Date().toISOString(),
            });
            const messageString = message.prepareMessage();
            const signature = await signMessageAsync({message: messageString});
            const res = await siweVerify(messageString, signature);
            setToken(res.token);
            setAuthedAddress(res.address);
        } catch (err) {
            setError(err instanceof Error ? err.message : String(err));
        } finally {
            setBusy(false);
        }
    };

    const signOut = () => {
        setToken(null);
        setAuthedAddress(null);
    };

    return (
        <section
            data-testid="siwe-panel"
            className="rounded-xl border border-zinc-800 bg-zinc-950/80 p-6 space-y-4"
        >
            <header className="flex items-center justify-between">
                <h2 className="text-lg font-semibold">Sign In With Ethereum</h2>
                <ConnectButton chainStatus="icon" showBalance={false} />
            </header>

            {!isConnected && (
                <p className="text-sm text-zinc-400">Connect a wallet to sign in.</p>
            )}

            {isConnected && !authedAddress && (
                <div className="space-y-3">
                    <p className="text-sm text-zinc-400">
                        Connected as <span className="mono">{address}</span>. Sign a SIWE message
                        to authenticate with the indexer.
                    </p>
                    <button
                        onClick={signIn}
                        disabled={busy}
                        data-testid="siwe-sign-in"
                        className="rounded-lg bg-emerald-500 hover:bg-emerald-400 disabled:bg-zinc-700 disabled:text-zinc-400 px-4 py-2 text-sm font-medium"
                    >
                        {busy ? "Signing…" : "Sign in"}
                    </button>
                </div>
            )}

            {authedAddress && (
                <div className="space-y-3">
                    <p className="text-sm">
                        <span className="text-zinc-400">Authenticated as: </span>
                        <span className="mono" data-testid="siwe-authed-address">
                            {authedAddress}
                        </span>
                    </p>
                    <button
                        onClick={signOut}
                        data-testid="siwe-sign-out"
                        className="rounded-lg bg-zinc-800 hover:bg-zinc-700 px-4 py-2 text-sm font-medium"
                    >
                        Sign out
                    </button>
                </div>
            )}

            {error && (
                <p className="text-xs text-rose-400" data-testid="siwe-error">
                    {error}
                </p>
            )}
        </section>
    );
}
