# BLS Threshold Cryptography Library

The blsttc library provides a high-performance implementation of BLS (Boneh-Lynn-Shacham) threshold signatures using the BLS12-381 curve. It enables secure distributed key generation, threshold signatures, and key aggregation.

## Overview

The library provides:

- BLS12-381 curve operations
- Threshold signature schemes
- Distributed key generation
- Key and signature aggregation
- Secure serialization
- Batch verification

## Installation

### Rust

```toml
[dependencies]
blsttc = "0.1.0"

# Optional features
blsttc = { version = "0.1.0", features = ["serde", "rayon"] }
```

### Python

```bash
pip install blsttc

# With optional features
pip install blsttc[serde,rayon]
```

## Basic Usage

### Simple Signing

```rust
// Rust
use blsttc::{SecretKey, PublicKey, Signature};

// Generate keys
let sk = SecretKey::random();
let pk = sk.public_key();

// Sign and verify
let msg = b"Hello, World!";
let sig = sk.sign(msg);
assert!(pk.verify(&sig, msg));
```

```python
# Python
from blsttc import SecretKey, PublicKey, Signature

# Generate keys
sk = SecretKey.random()
pk = sk.public_key()

# Sign and verify
msg = b"Hello, World!"
sig = sk.sign(msg)
assert pk.verify(sig, msg)
```

## Advanced Usage

### Threshold Signatures

```rust
// Rust
use blsttc::{SecretKeySet, PublicKeySet};

// Generate threshold key set
let threshold = 2;  // t-of-n, where t = threshold + 1
let sk_set = SecretKeySet::random(threshold, &mut rng);
let pk_set = sk_set.public_keys();

// Generate key shares
let sk_shares: Vec<_> = (0..=threshold + 1)
    .map(|i| sk_set.secret_key_share(i))
    .collect();
let pk_shares: Vec<_> = (0..=threshold + 1)
    .map(|i| pk_set.public_key_share(i))
    .collect();

// Sign with shares
let msg = b"Hello, World!";
let sig_shares: Vec<_> = sk_shares.iter()
    .map(|sk| sk.sign(msg))
    .collect();

// Combine signatures
let sig = pk_set.combine_signatures(&sig_shares[..threshold + 1])?;
assert!(pk_set.public_key().verify(&sig, msg));
```

```python
# Python
from blsttc import SecretKeySet, PublicKeySet

# Generate threshold key set
threshold = 2  # t-of-n, where t = threshold + 1
sk_set = SecretKeySet.random(threshold)
pk_set = sk_set.public_keys()

# Generate key shares
sk_shares = [
    sk_set.secret_key_share(i)
    for i in range(threshold + 2)
]
pk_shares = [
    pk_set.public_key_share(i)
    for i in range(threshold + 2)
]

# Sign with shares
msg = b"Hello, World!"
sig_shares = [sk.sign(msg) for sk in sk_shares]

# Combine signatures
sig = pk_set.combine_signatures(sig_shares[:threshold + 1])
assert pk_set.public_key().verify(sig, msg)
```

### Distributed Key Generation

```rust
// Rust
use blsttc::dkg::{DKGParticipant, Contribution};

// Initialize participants
let n_participants = 5;
let threshold = 2;
let mut participants: Vec<_> = (0..n_participants)
    .map(|i| DKGParticipant::new(i, threshold, n_participants))
    .collect();

// Round 1: Generate and share contributions
let contributions: Vec<_> = participants.iter_mut()
    .map(|p| p.generate_contribution())
    .collect();

// Share contributions with all participants
for p in &mut participants {
    for c in &contributions {
        p.handle_contribution(c)?;
    }
}

// Round 2: Generate complaints/justifications
let complaints: Vec<_> = participants.iter_mut()
    .map(|p| p.generate_complaints())
    .collect();

// Handle complaints
for p in &mut participants {
    for c in &complaints {
        p.handle_complaint(c)?;
    }
}

// Finalize keys
let key_sets: Vec<_> = participants.iter_mut()
    .map(|p| p.finalize())
    .collect::<Result<_, _>>()?;
```

```python
# Python
from blsttc.dkg import DKGParticipant, Contribution

# Initialize participants
n_participants = 5
threshold = 2
participants = [
    DKGParticipant(i, threshold, n_participants)
    for i in range(n_participants)
]

# Round 1: Generate and share contributions
contributions = [
    p.generate_contribution()
    for p in participants
]

# Share contributions with all participants
for p in participants:
    for c in contributions:
        p.handle_contribution(c)

# Round 2: Generate complaints/justifications
complaints = [
    p.generate_complaints()
    for p in participants
]

# Handle complaints
for p in participants:
    for c in complaints:
        p.handle_complaint(c)

# Finalize keys
key_sets = [p.finalize() for p in participants]
```

### Batch Verification

```rust
// Rust
use blsttc::batch::{BatchVerifier, VerificationStrategy};

// Create batch verifier
let mut verifier = BatchVerifier::new(VerificationStrategy::Strict);

// Add signatures to batch
for (sig, msg, pk) in signatures {
    verifier.queue((sig, msg, pk));
}

// Verify all signatures
assert!(verifier.verify()?);
```

```python
# Python
from blsttc.batch import BatchVerifier, VerificationStrategy

# Create batch verifier
verifier = BatchVerifier(VerificationStrategy.STRICT)

# Add signatures to batch
for sig, msg, pk in signatures:
    verifier.queue((sig, msg, pk))

# Verify all signatures
assert verifier.verify()
```

### Serialization

```rust
// Rust
use blsttc::serde::{serialize, deserialize};

// Serialize keys and signatures
let sk_bytes = serialize(&sk)?;
let pk_bytes = serialize(&pk)?;
let sig_bytes = serialize(&sig)?;

// Deserialize
let sk: SecretKey = deserialize(&sk_bytes)?;
let pk: PublicKey = deserialize(&pk_bytes)?;
let sig: Signature = deserialize(&sig_bytes)?;
```

```python
# Python
from blsttc.serde import serialize, deserialize

# Serialize keys and signatures
sk_bytes = serialize(sk)
pk_bytes = serialize(pk)
sig_bytes = serialize(sig)

# Deserialize
sk = deserialize(SecretKey, sk_bytes)
pk = deserialize(PublicKey, pk_bytes)
sig = deserialize(Signature, sig_bytes)
```

## Performance Optimization

### Parallel Processing

```rust
// Rust
use blsttc::parallel::{ParallelVerifier, ThreadPool};

// Create thread pool
let pool = ThreadPool::new(4);  // 4 threads

// Verify signatures in parallel
let results = pool.verify_batch(&signatures)?;
```

```python
# Python
from blsttc.parallel import ParallelVerifier, ThreadPool

# Create thread pool
pool = ThreadPool(4)  # 4 threads

# Verify signatures in parallel
results = pool.verify_batch(signatures)
```

### Memory Management

```rust
// Rust
use blsttc::memory::{MemoryConfig, CacheConfig};

// Configure memory usage
let config = MemoryConfig {
    max_batch_size: 1000,
    max_cache_size: 100 * 1024 * 1024,  // 100MB
};

// Use configuration
let verifier = BatchVerifier::with_config(config);
```

```python
# Python
from blsttc.memory import MemoryConfig, CacheConfig

# Configure memory usage
config = MemoryConfig(
    max_batch_size=1000,
    max_cache_size=100 * 1024 * 1024  # 100MB
)

# Use configuration
verifier = BatchVerifier(config=config)
```

## Security Considerations

### Key Generation

- Use cryptographically secure random number generation
- Protect secret keys and shares
- Validate all inputs
- Use appropriate thresholds

### Signature Verification

- Validate public keys
- Check signature validity
- Use batch verification carefully
- Handle errors appropriately

## Error Handling

```rust
// Rust
use blsttc::error::{Error, Result};

match sk_set.combine_signatures(&signatures) {
    Ok(sig) => {
        // Combined signature
    }
    Err(Error::InvalidShare(e)) => {
        // Handle invalid share
    }
    Err(Error::NotEnoughShares(e)) => {
        // Handle insufficient shares
    }
    Err(e) => {
        // Handle other errors
    }
}
```

```python
# Python
from blsttc.error import (
    Error, InvalidShareError,
    NotEnoughSharesError
)

try:
    sig = sk_set.combine_signatures(signatures)
except InvalidShareError as e:
    # Handle invalid share
except NotEnoughSharesError as e:
    # Handle insufficient shares
except Error as e:
    # Handle other errors
```

## Best Practices

1. **Key Management**
   - Secure key generation
   - Safe key storage
   - Regular key rotation
   - Proper share distribution

2. **Performance**
   - Use batch verification
   - Enable parallel processing
   - Configure appropriate batch sizes
   - Monitor memory usage

3. **Security**
   - Validate all inputs
   - Use secure random numbers
   - Handle errors appropriately
   - Follow cryptographic best practices

4. **Integration**
   - Use appropriate thresholds
   - Implement proper error handling
   - Monitor system performance
   - Regular security audits

## API Reference

See the complete [API Reference](https://docs.rs/blsttc) for detailed documentation of all types and functions.
