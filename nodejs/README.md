# Autonomi Node.js Client

TypeScript/JavaScript bindings for the Autonomi client.

## Installation

```bash
npm install @autonomi/client
```

## Usage

```typescript
import { Client } from '@autonomi/client';

async function example() {
  // Connect to the network
  const client = await Client.connect({
    peers: ['/ip4/127.0.0.1/tcp/12000']
  });

  // Create a payment option using a wallet
  const payment = {
    type: 'wallet' as const,
    wallet: 'your_wallet_address'
  };

  // Upload public data
  const data = Buffer.from('Hello, Safe Network!');
  const addr = await client.dataPutPublic(data, payment);
  console.log(`Data uploaded to: ${addr}`);

  // Download public data
  const retrieved = await client.dataGetPublic(addr);
  console.log(`Retrieved: ${retrieved.toString()}`);
}
```

## Features

- TypeScript support with full type definitions
- Async/await API
- Support for:
  - Public and private data operations
  - Linked Lists
  - Pointers
  - Vaults
  - User data management

## API Reference

### Client

The main interface to interact with the Autonomi network.

#### Connection

```typescript
static connect(config: NetworkConfig): Promise<Client>
```

#### Data Operations

```typescript
dataPutPublic(data: Buffer, payment: PaymentOption): Promise<string>
dataGetPublic(address: string): Promise<Buffer>
```

#### Linked List Operations

```typescript
linkedListGet(address: string): Promise<any[]>
linkedListPut(options: LinkedListOptions, payment: PaymentOption): Promise<void>
linkedListCost(key: string): Promise<string>
```

#### Pointer Operations

```typescript
pointerGet(address: string): Promise<any>
pointerPut(options: PointerOptions, payment: PaymentOption): Promise<void>
pointerCost(key: string): Promise<string>
```

#### Vault Operations

```typescript
vaultCost(key: string): Promise<string>
writeBytesToVault(data: Buffer, payment: PaymentOption, options: VaultOptions): Promise<string>
fetchAndDecryptVault(key: string): Promise<[Buffer, number]>
getUserDataFromVault(key: string): Promise<UserData>
putUserDataToVault(key: string, payment: PaymentOption, userData: UserData): Promise<void>
```

## Development

```bash
# Install dependencies
npm install

# Build
npm run build

# Run tests
npm test

# Run tests in watch mode
npm run test:watch

# Lint
npm run lint
```

## License

GPL-3.0
