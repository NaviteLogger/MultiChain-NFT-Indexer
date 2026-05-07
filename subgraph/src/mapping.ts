import {Address, BigInt} from "@graphprotocol/graph-ts";
import {
    Transfer as TransferEvent,
    Minted as MintedEvent,
} from "../generated/IndexedNFT/IndexedNFT";
import {Holder, Token, Transfer} from "../generated/schema";

const ZERO_ADDRESS = Address.zero();

export function handleTransfer(event: TransferEvent): void {
    const tokenKey =
        event.address.toHexString() + "-" + event.params.tokenId.toString();

    let token = Token.load(tokenKey);
    if (token == null) {
        token = new Token(tokenKey);
        token.contract = event.address;
        token.tokenId = event.params.tokenId;
        token.mintBlock = event.block.number;
    }
    token.currentOwner = event.params.to;
    token.lastTransferBlock = event.block.number;
    token.save();

    const xfer = new Transfer(
        event.transaction.hash.toHexString() + "-" + event.logIndex.toString(),
    );
    xfer.token = tokenKey;
    xfer.from = event.params.from;
    xfer.to = event.params.to;
    xfer.blockNumber = event.block.number;
    xfer.blockTimestamp = event.block.timestamp;
    xfer.txHash = event.transaction.hash;
    xfer.logIndex = event.logIndex;
    xfer.save();

    if (event.params.from.notEqual(ZERO_ADDRESS)) {
        const fromHolder = upsertHolder(event.params.from);
        fromHolder.tokenCount = fromHolder.tokenCount - 1;
        fromHolder.save();
    }
    const toHolder = upsertHolder(event.params.to);
    toHolder.tokenCount = toHolder.tokenCount + 1;
    toHolder.save();
}

export function handleMinted(event: MintedEvent): void {
    const tokenKey =
        event.address.toHexString() + "-" + event.params.tokenId.toString();
    let token = Token.load(tokenKey);
    if (token == null) {
        token = new Token(tokenKey);
        token.contract = event.address;
        token.tokenId = event.params.tokenId;
        token.currentOwner = event.params.to;
        token.mintBlock = event.block.number;
        token.lastTransferBlock = event.block.number;
    }
    token.uri = event.params.uri;
    token.save();
}

function upsertHolder(address: Address): Holder {
    const id = address.toHexString();
    let holder = Holder.load(id);
    if (holder == null) {
        holder = new Holder(id);
        holder.tokenCount = 0;
    }
    return holder;
}
