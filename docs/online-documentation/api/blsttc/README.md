# BLS Threshold Crypto API Reference

BLS Threshold Crypto (blsttc) is a Rust implementation of BLS (Boneh-Lynn-Shacham) threshold signatures with support for both Rust and Python interfaces.

## Installation

=== "Python"
    ```bash
    # Install using uv (recommended)
    curl -LsSf <https://astral.sh/uv/install.sh> | sh
    uv pip install blsttc

    # Or using pip
    pip install blsttc
    ```

=== "Rust"
    ```toml
    # Add to Cargo.toml
    [dependencies]
    blsttc = "8.0.2"
    ```

## Basic Usage

=== "Python"
    ```python
    from blsttc import SecretKey, PublicKey, Signature

    # Generate a secret key
    secret_key = SecretKey.random()

    # Get the corresponding public key
    public_key = secret_key.public_key()

    # Sign a message
    message = b"Hello, World!"
    signature = secret_key.sign(message)

    # Verify the signature
    assert public_key.verify(signature, message)
    ```

=== "Rust"
    ```rust
    use blsttc::{SecretKey, PublicKey, Signature};

    // Generate a secret key
    let secret_key = SecretKey::random();

    // Get the corresponding public key
    let public_key = secret_key.public_key();

    // Sign a message
    let message = b"Hello, World!";
    let signature = secret_key.sign(message);

    // Verify the signature
    assert!(public_key.verify(&signature, message));
    ```

## Threshold Signatures

=== "Python"
    ```python
    from blsttc import SecretKeySet, PublicKeySet

    # Create a threshold signature scheme
    threshold = 3  # Minimum signatures required
    total = 5      # Total number of shares
    sk_set = SecretKeySet.random(threshold)

    # Get the public key set
    pk_set = sk_set.public_keys()

    # Generate secret key shares
    secret_shares = [sk_set.secret_key_share(i) for i in range(total)]

    # Sign with individual shares
    message = b"Hello, World!"
    sig_shares = [share.sign(message) for share in secret_shares]

    # Combine signatures
    combined_sig = pk_set.combine_signatures(sig_shares[:threshold])

    # Verify the combined signature
    assert pk_set.public_key().verify(combined_sig, message)
    ```

=== "Rust"
    ```rust
    use blsttc::{SecretKeySet, PublicKeySet};

    // Create a threshold signature scheme
    let threshold = 3;  // Minimum signatures required
    let total = 5;      // Total number of shares
    let sk_set = SecretKeySet::random(threshold);

    // Get the public key set
    let pk_set = sk_set.public_keys();

    // Generate secret key shares
    let secret_shares: Vec<_> = (0..total)
        .map(|i| sk_set.secret_key_share(i))
        .collect();

    // Sign with individual shares
    let message = b"Hello, World!";
    let sig_shares: Vec<_> = secret_shares
        .iter()
        .map(|share| share.sign(message))
        .collect();

    // Combine signatures
    let combined_sig = pk_set.combine_signatures(sig_shares[..threshold].iter())?;

    // Verify the combined signature
    assert!(pk_set.public_key().verify(&combined_sig, message));
    ```

## Advanced Features

### Key Generation

=== "Python"
    ```python
    from blsttc import SecretKey, Fr

    # Generate from random seed
    secret_key = SecretKey.random()

    # Generate from bytes
    bytes_data = b"some-32-byte-seed"
    secret_key = SecretKey.from_bytes(bytes_data)

    # Generate from field element
    fr = Fr.random()
    secret_key = SecretKey.from_fr(fr)
    ```

=== "Rust"
    ```rust
    use blsttc::{SecretKey, Fr};
    use rand::thread_rng;

    // Generate from random seed
    let secret_key = SecretKey::random();

    // Generate from bytes
    let bytes_data = b"some-32-byte-seed";
    let secret_key = SecretKey::from_bytes(bytes_data)?;

    // Generate from field element
    let fr = Fr::random();
    let secret_key = SecretKey::from_fr(&fr);
    ```

### Serialization

=== "Python"
    ```python
    # Serialize keys and signatures
    sk_bytes = secret_key.to_bytes()
    pk_bytes = public_key.to_bytes()
    sig_bytes = signature.to_bytes()

    # Deserialize
    sk = SecretKey.from_bytes(sk_bytes)
    pk = PublicKey.from_bytes(pk_bytes)
    sig = Signature.from_bytes(sig_bytes)
    ```

=== "Rust"
    ```rust
    // Serialize keys and signatures
    let sk_bytes = secret_key.to_bytes();
    let pk_bytes = public_key.to_bytes();
    let sig_bytes = signature.to_bytes();

    // Deserialize
    let sk = SecretKey::from_bytes(&sk_bytes)?;
    let pk = PublicKey::from_bytes(&pk_bytes)?;
    let sig = Signature::from_bytes(&sig_bytes)?;
    ```

## Error Handling

=== "Python"
    ```python
    try:
        # Operations that might fail
        sk = SecretKey.from_bytes(invalid_bytes)
    except ValueError as e:
        print(f"Invalid key bytes: {e}")

    try:
        # Signature verification
        if not pk.verify(sig, msg):
            print("Invalid signature")
    except Exception as e:
        print(f"Verification error: {e}")
    ```

=== "Rust"
    ```rust
    use blsttc::error::Error;

    // Handle key generation errors
    match SecretKey::from_bytes(invalid_bytes) {
        Ok(sk) => println!("Key generated successfully"),
        Err(Error::InvalidBytes) => println!("Invalid key bytes"),
        Err(e) => println!("Other error: {}", e),
    }

    // Handle signature verification
    if !pk.verify(&sig, msg) {
        println!("Invalid signature");
    }
    ```

## Best Practices

1. **Key Management**
   - Securely store private keys
   - Use strong random number generation
   - Implement key rotation policies

2. **Threshold Selection**
   - Choose appropriate threshold values
   - Consider fault tolerance requirements
   - Balance security and availability

3. **Performance**
   - Cache public keys when possible
   - Batch verify signatures when possible
   - Use appropriate buffer sizes

4. **Security**
   - Validate all inputs
   - Use secure random number generation
   - Implement proper error handling

## Common Use Cases

1. **Distributed Key Generation**
   - Generate keys for distributed systems
   - Share keys among multiple parties
   - Implement threshold cryptography

2. **Signature Aggregation**
   - Combine multiple signatures
   - Reduce signature size
   - Improve verification efficiency

3. **Consensus Protocols**
   - Implement Byzantine fault tolerance
   - Create distributed voting systems
   - Build secure multiparty computation
