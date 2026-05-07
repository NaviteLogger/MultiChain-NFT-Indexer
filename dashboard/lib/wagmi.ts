import {getDefaultConfig} from "@rainbow-me/rainbowkit";
import {sepolia, baseSepolia, polygonAmoy, mainnet} from "wagmi/chains";
import {http} from "wagmi";

const projectId = process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || "demo";

export const wagmiConfig = getDefaultConfig({
    appName: "MultiChain NFT Indexer",
    projectId,
    chains: [sepolia, baseSepolia, polygonAmoy, mainnet],
    transports: {
        [sepolia.id]: http(),
        [baseSepolia.id]: http(),
        [polygonAmoy.id]: http(),
        [mainnet.id]: http(),
    },
    ssr: true,
});
