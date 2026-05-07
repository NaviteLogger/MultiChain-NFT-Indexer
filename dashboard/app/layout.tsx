import type {Metadata} from "next";
import "./globals.css";
import Providers from "./providers";

export const metadata: Metadata = {
    title: "MultiChain NFT Indexer",
    description: "Live event feed and SIWE-authenticated dashboard for the indexer.",
};

export default function RootLayout({children}: {children: React.ReactNode}) {
    return (
        <html lang="en">
            <body>
                <Providers>{children}</Providers>
            </body>
        </html>
    );
}
