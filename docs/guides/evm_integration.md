# EVM Integration Guide

This guide explains how to integrate Autonomi with EVM-compatible networks for testing and development.

## Supported Networks

- Local Hardhat network
- Sepolia testnet
- Goerli testnet
- Custom EVM networks

## Setting Up Test Networks

### Local Hardhat Network

```bash
npx hardhat node
```

### Connecting to Test Networks

```typescript
import { EvmNetwork } from '@autonomi/client';

const network = new EvmNetwork({
  chainId: 31337, // Local hardhat network
  rpcUrl: 'http://127.0.0.1:8545'
});
```

## Deploying Test Contracts

1. Compile contracts
2. Deploy using Hardhat
3. Interact with contracts

## Testing with Different Networks

- Network configuration
- Gas settings
- Contract deployment
- Transaction handling

## Best Practices

- Error handling
- Gas optimization
- Security considerations
- Testing strategies
