export type Address = string;
export type HexString = string;
export type SecretKey = string;
export type PublicKey = string;

export interface PaymentOption {
    type: 'wallet';
    wallet: string;
}

export interface GraphEntryOptions {
    owner: PublicKey;
    counter: number;
    target: string;
    key: SecretKey;
}

export interface PointerOptions {
    owner: PublicKey;
    counter: number;
    target: string;
    key: SecretKey;
}

export interface VaultOptions {
    key: SecretKey;
    contentType?: number;
}

export interface UserData {
    fileArchives: Array<[string, string]>;
    privateFileArchives: Array<[string, string]>;
}

export interface NetworkConfig {
    peers: string[];
    network?: 'arbitrum' | 'arbitrum_testnet';
} 