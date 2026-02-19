# 69. Cryptography and Hashing in Rust

- **`std` Hasher trait** — `Hash`, `Hasher`, `DefaultHasher`, and plugging in custom hashers for `HashMap`
- **Non-cryptographic hashing** — FxHash, xxHash, ahash, and HashDoS protection
- **RustCrypto crates** — SHA-2, SHA-3, BLAKE3, and HMAC with the `Digest` trait pattern
- **The `ring` crate** — hashing, HMAC, and X25519 key agreement via its high-assurance API
- **Symmetric encryption** — AES-256-GCM and ChaCha20-Poly1305 (AEAD only)
- **Asymmetric signatures** — Ed25519 with `ed25519-dalek`
- **Password hashing** — Argon2 and bcrypt, with correct verify patterns
- **Cryptographic vs. non-cryptographic hashes** — a comparison table and decision guide
- **Secure coding practices** — constant-time comparison (`subtle`), memory zeroization (`zeroize`), nonce uniqueness, avoiding DIY crypto, and supply-chain auditing with `cargo audit`

Rust has emerged as a strong language for implementing cryptographic systems due to its memory safety guarantees, zero-cost abstractions, and absence of undefined behavior. This document covers the major crates and patterns used for cryptography and hashing in Rust, the distinction between cryptographic and non-cryptographic hashes, and secure coding practices.

---

## Table of Contents

1. [The Rust Cryptography Ecosystem](#1-the-rust-cryptography-ecosystem)
2. [The `std` Hasher Trait](#2-the-std-hasher-trait)
3. [Non-Cryptographic Hashing](#3-non-cryptographic-hashing)
4. [Cryptographic Hashing with RustCrypto](#4-cryptographic-hashing-with-rustcrypto)
5. [The `ring` Crate](#5-the-ring-crate)
6. [Symmetric Encryption](#6-symmetric-encryption)
7. [Asymmetric Cryptography and Signatures](#7-asymmetric-cryptography-and-signatures)
8. [Password Hashing](#8-password-hashing)
9. [Cryptographic vs. Non-Cryptographic Hashes](#9-cryptographic-vs-non-cryptographic-hashes)
10. [Secure Coding Practices](#10-secure-coding-practices)
11. [Summary Table](#11-summary-table)

---

## 1. The Rust Cryptography Ecosystem

Rust's cryptographic landscape is organized around two main pillars:

**RustCrypto** is a GitHub organization (`https://github.com/RustCrypto`) that maintains a large collection of pure-Rust cryptographic primitives. Each algorithm is a separate crate, enabling minimal dependency footprints. Crates include `sha2`, `sha3`, `aes`, `chacha20poly1305`, `rsa`, `p256`, `argon2`, `hmac`, and many more.

**`ring`** is a Rust wrapper around the well-audited BoringSSL cryptographic library. It prioritizes correctness and performance over API flexibility, making it an excellent choice for production TLS and similar workloads.

Other notable crates include `rustls` (TLS), `openssl` (OpenSSL bindings), `dalek-cryptography` (Ed25519, X25519), and `sodiumoxide` (libsodium bindings).

---

## 2. The `std` Hasher Trait

Rust's standard library defines the `Hasher` and `Hash` traits in `std::hash`. These traits power `HashMap` and `HashSet` and are designed for **fast, non-cryptographic** hashing.

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn calculate_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn main() {
    let x = "hello world";
    let y = 42u32;

    println!("Hash of {:?}: {}", x, calculate_hash(&x));
    println!("Hash of {}: {}", y, calculate_hash(&y));
}
```

### Implementing `Hash` for Custom Types

```rust
use std::hash::{Hash, Hasher};

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

// Usually you'd just derive it:
#[derive(Debug, Hash, PartialEq, Eq)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}
```

### Using a Custom Hasher with HashMap

You can swap out the hasher used by `HashMap` for better performance or security:

```rust
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

// Example: using FxHasher from the `rustc-hash` crate for fast hashing
// Add to Cargo.toml: rustc-hash = "1"
use rustc_hash::FxHashMap;

fn main() {
    let mut map: FxHashMap<String, i32> = FxHashMap::default();
    map.insert("foo".to_string(), 1);
    map.insert("bar".to_string(), 2);
    println!("{:?}", map.get("foo")); // Some(1)
}
```

> **Important:** `DefaultHasher` is not guaranteed to be stable across Rust versions or processes. Never use it to persist hashes to disk or compare hashes across processes.

---

## 3. Non-Cryptographic Hashing

Non-cryptographic hash functions are optimized purely for **speed and low collision rates** in hash tables. They make no security guarantees — an attacker who can control inputs can engineer hash collisions.

### Popular Non-Cryptographic Hashers

| Crate | Algorithm | Use Case |
|---|---|---|
| `rustc-hash` | FxHash | Compiler internals, very fast |
| `ahash` | AHash | Default in `hashbrown` (used by std) |
| `twox-hash` | xxHash | Large data, streaming |
| `fnv` | FNV-1a | Small keys, simple |

```toml
# Cargo.toml
[dependencies]
twox-hash = "1"
```

```rust
use std::hash::BuildHasherDefault;
use std::collections::HashMap;
use twox_hash::XxHash64;

fn main() {
    let mut map: HashMap<&str, u32, BuildHasherDefault<XxHash64>> =
        HashMap::with_hasher(BuildHasherDefault::default());

    map.insert("alpha", 1);
    map.insert("beta", 2);

    for (k, v) in &map {
        println!("{}: {}", k, v);
    }
}
```

### HashDoS Protection

The `ahash` crate (used internally by Rust's `HashMap`) includes **randomized seeding** to prevent HashDoS attacks where an adversary crafts colliding keys to degrade performance to O(n).

```rust
// HashMap in std already uses ahash with random seeds by default
use std::collections::HashMap;

let mut safe_map: HashMap<String, String> = HashMap::new();
safe_map.insert("key".into(), "value".into());
// Immune to HashDoS because the seed is randomized per-process
```

---

## 4. Cryptographic Hashing with RustCrypto

Cryptographic hash functions must satisfy: **pre-image resistance**, **second pre-image resistance**, and **collision resistance**. The RustCrypto crates implement them via the `Digest` trait from the `digest` crate.

```toml
# Cargo.toml
[dependencies]
sha2 = "0.10"
sha3 = "0.10"
digest = "0.10"
hex = "0.4"
```

### SHA-256

```rust
use sha2::{Sha256, Digest};

fn main() {
    let mut hasher = Sha256::new();
    hasher.update(b"Hello, ");
    hasher.update(b"world!");
    let result = hasher.finalize();
    println!("SHA-256: {:x}", result);
    // SHA-256: 4ae7c3b6ac0beff671efa8cf57386151c06e58ca53a78d83f36107316cec125f
}
```

### SHA-3 (Keccak)

```rust
use sha3::{Sha3_256, Digest};

fn hash_file_contents(data: &[u8]) -> String {
    let hash = Sha3_256::digest(data);
    format!("{:x}", hash)
}

fn main() {
    let data = b"sensitive data";
    println!("SHA3-256: {}", hash_file_contents(data));
}
```

### BLAKE3

BLAKE3 is a modern hash function that is fast, parallelizable, and cryptographically secure. It is not part of RustCrypto but has its own well-maintained crate.

```toml
[dependencies]
blake3 = "1"
```

```rust
fn main() {
    let hash = blake3::hash(b"Hello, world!");
    println!("BLAKE3: {}", hash.to_hex());

    // Keyed hashing (MAC-like without HMAC overhead)
    let key = [0u8; 32];
    let keyed = blake3::keyed_hash(&key, b"message");
    println!("BLAKE3 keyed: {}", keyed.to_hex());

    // Key derivation
    let derived = blake3::derive_key("app context v1", b"input key material");
    println!("Derived key: {}", hex::encode(derived));
}
```

### HMAC (Hash-based Message Authentication Code)

HMAC proves both integrity and authenticity using a shared secret key.

```toml
[dependencies]
hmac = "0.12"
sha2 = "0.10"
```

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn compute_hmac(key: &[u8], message: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC can take key of any size");
    mac.update(message);
    mac.finalize().into_bytes().to_vec()
}

fn verify_hmac(key: &[u8], message: &[u8], expected_tag: &[u8]) -> bool {
    let mut mac = HmacSha256::new_from_slice(key).unwrap();
    mac.update(message);
    // Use constant-time verify to prevent timing attacks
    mac.verify_slice(expected_tag).is_ok()
}

fn main() {
    let key = b"super-secret-key";
    let message = b"important payload";

    let tag = compute_hmac(key, message);
    println!("HMAC-SHA256: {}", hex::encode(&tag));

    let valid = verify_hmac(key, message, &tag);
    println!("Valid: {}", valid); // true

    let tampered = verify_hmac(key, b"tampered payload", &tag);
    println!("Tampered: {}", tampered); // false
}
```

---

## 5. The `ring` Crate

`ring` provides a curated, high-assurance API built on BoringSSL. It does not expose unsafe primitives and enforces correct usage patterns at the API level.

```toml
[dependencies]
ring = "0.17"
```

### Hashing with `ring`

```rust
use ring::digest;

fn main() {
    let data = b"The quick brown fox";

    let sha256 = digest::digest(&digest::SHA256, data);
    println!("SHA-256: {}", hex::encode(sha256.as_ref()));

    let sha512 = digest::digest(&digest::SHA512, data);
    println!("SHA-512: {}", hex::encode(sha512.as_ref()));
}
```

### HMAC with `ring`

```rust
use ring::{hmac, rand};
use ring::rand::SecureRandom;

fn main() {
    let rng = rand::SystemRandom::new();
    let mut key_bytes = [0u8; 32];
    rng.fill(&mut key_bytes).expect("Failed to generate key");

    let key = hmac::Key::new(hmac::HMAC_SHA256, &key_bytes);
    let tag = hmac::sign(&key, b"authenticated message");

    // Verify
    hmac::verify(&key, b"authenticated message", tag.as_ref())
        .expect("HMAC verification failed");
    println!("HMAC verified successfully!");
}
```

### Key Agreement with X25519 (ECDH)

```rust
use ring::agreement;
use ring::rand::SystemRandom;

fn main() {
    let rng = SystemRandom::new();

    // Alice generates her ephemeral key pair
    let alice_private = agreement::EphemeralPrivateKey::generate(
        &agreement::X25519,
        &rng,
    ).unwrap();
    let alice_public = alice_private.compute_public_key().unwrap();

    // Bob generates his ephemeral key pair
    let bob_private = agreement::EphemeralPrivateKey::generate(
        &agreement::X25519,
        &rng,
    ).unwrap();
    let bob_public = bob_private.compute_public_key().unwrap();

    // Alice computes the shared secret
    let alice_secret = agreement::agree_ephemeral(
        alice_private,
        &agreement::UnparsedPublicKey::new(&agreement::X25519, bob_public.as_ref()),
        |key_material| key_material.to_vec(),
    ).unwrap();

    // Bob computes the shared secret
    let bob_secret = agreement::agree_ephemeral(
        bob_private,
        &agreement::UnparsedPublicKey::new(&agreement::X25519, alice_public.as_ref()),
        |key_material| key_material.to_vec(),
    ).unwrap();

    assert_eq!(alice_secret, bob_secret);
    println!("Shared secret established: {} bytes", alice_secret.len());
}
```

---

## 6. Symmetric Encryption

### AES-GCM (Authenticated Encryption)

AES-GCM is the recommended symmetric cipher — it provides both confidentiality and integrity.

```toml
[dependencies]
aes-gcm = "0.10"
rand = "0.8"
```

```rust
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};

fn encrypt(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bit nonce
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .expect("Encryption failed");
    (nonce.to_vec(), ciphertext)
}

fn decrypt(key: &Key<Aes256Gcm>, nonce_bytes: &[u8], ciphertext: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .expect("Decryption failed — data may be tampered")
}

fn main() {
    let key = Aes256Gcm::generate_key(OsRng);
    let plaintext = b"Top secret message";

    let (nonce, ciphertext) = encrypt(&key, plaintext);
    let recovered = decrypt(&key, &nonce, &ciphertext);

    assert_eq!(plaintext, recovered.as_slice());
    println!("Round-trip successful: {:?}", std::str::from_utf8(&recovered).unwrap());
}
```

### ChaCha20-Poly1305

ChaCha20-Poly1305 is preferred on systems without hardware AES acceleration (e.g., mobile).

```toml
[dependencies]
chacha20poly1305 = "0.10"
```

```rust
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305,
};

fn main() {
    let key = ChaCha20Poly1305::generate_key(&mut OsRng);
    let cipher = ChaCha20Poly1305::new(&key);
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

    let ciphertext = cipher.encrypt(&nonce, b"plaintext message".as_ref()).unwrap();
    let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref()).unwrap();

    println!("Decrypted: {}", String::from_utf8(plaintext).unwrap());
}
```

---

## 7. Asymmetric Cryptography and Signatures

### Ed25519 Digital Signatures

Ed25519 is the recommended signature scheme: fast, secure, and with a small key size.

```toml
[dependencies]
ed25519-dalek = "2"
rand = "0.8"
```

```rust
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

fn main() {
    // Key generation
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key: VerifyingKey = signing_key.verifying_key();

    // Signing
    let message = b"Important document content";
    let signature = signing_key.sign(message);
    println!("Signature: {}", hex::encode(signature.to_bytes()));

    // Verification
    verifying_key.verify(message, &signature)
        .expect("Signature is valid");
    println!("Signature verified!");

    // Tampered message should fail
    let tampered = b"Tampered document content";
    assert!(verifying_key.verify(tampered, &signature).is_err());
    println!("Correctly rejected tampered message.");
}
```

---

## 8. Password Hashing

Never store raw passwords — always use a **password hashing function** designed to be slow and memory-hard.

### Argon2

Argon2 is the winner of the Password Hashing Competition and is the current best practice.

```toml
[dependencies]
argon2 = "0.5"
password-hash = "0.5"
rand_core = { version = "0.6", features = ["getrandom"] }
```

```rust
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};

fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Hashing failed")
        .to_string()
}

fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).expect("Invalid hash string");
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

fn main() {
    let password = "my-secret-password";
    let hash = hash_password(password);
    println!("Argon2 hash: {}", hash);

    println!("Correct password: {}", verify_password(password, &hash));
    println!("Wrong password: {}", verify_password("wrong", &hash));
}
```

### bcrypt

bcrypt is widely supported and still considered acceptable, though Argon2 is preferred for new systems.

```toml
[dependencies]
bcrypt = "0.15"
```

```rust
use bcrypt::{hash, verify, DEFAULT_COST};

fn main() {
    let password = "hunter2";
    let hashed = hash(password, DEFAULT_COST).unwrap();
    println!("bcrypt: {}", hashed);

    assert!(verify(password, &hashed).unwrap());
    println!("Password verified!");
}
```

---

## 9. Cryptographic vs. Non-Cryptographic Hashes

This is one of the most important distinctions in the field.

| Property | Non-Cryptographic (e.g., FxHash, xxHash) | Cryptographic (e.g., SHA-256, BLAKE3) |
|---|---|---|
| **Speed** | Extremely fast (GB/s) | Slower (100–500 MB/s typical) |
| **Collision resistance** | Weak — collisions can be engineered | Strong — computationally infeasible |
| **Pre-image resistance** | None — easily invertible | Strong — cannot reverse hash to input |
| **Output size** | 32 or 64 bits typical | 256–512 bits typical |
| **Seeded randomization** | Sometimes (ahash) | Not applicable |
| **Use case** | Hash maps, deduplication, caching | Integrity, authentication, passwords |
| **HashDoS vulnerability** | Yes (without random seed) | Not applicable |

### Decision Guide

```
Need to store in a HashMap?
  → Use std HashMap (ahash) or FxHashMap

Need to verify file integrity?
  → Use SHA-256 or BLAKE3

Need to store a password?
  → Use Argon2 (never SHA or BLAKE directly)

Need to authenticate a message?
  → Use HMAC-SHA256 or BLAKE3 keyed hash

Need to sign a document?
  → Use Ed25519

Need end-to-end encryption?
  → Use AES-256-GCM or ChaCha20-Poly1305
```

---

## 10. Secure Coding Practices

### 1. Use Constant-Time Comparisons

Comparing secret values with `==` leaks timing information. Always use constant-time equality for secrets.

```rust
use subtle::ConstantTimeEq;

fn secure_compare(a: &[u8], b: &[u8]) -> bool {
    // Returns Choice (not bool) to prevent the compiler from short-circuiting
    a.ct_eq(b).into()
}

// WRONG — timing side-channel
fn insecure_compare(token_a: &str, token_b: &str) -> bool {
    token_a == token_b // Leaks how many bytes match
}
```

### 2. Zeroize Sensitive Data

Use the `zeroize` crate to wipe secret keys from memory when they are no longer needed.

```toml
[dependencies]
zeroize = { version = "1", features = ["derive"] }
```

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
struct SecretKey {
    bytes: [u8; 32],
}

fn main() {
    let mut key = SecretKey { bytes: [0xAB; 32] };
    // use key...
    key.zeroize(); // Explicitly wipe; also called automatically on drop
    // key.bytes is now all zeros
}
```

### 3. Always Use Authenticated Encryption

Unauthenticated encryption (e.g., raw AES-CBC) is vulnerable to padding oracle and bit-flipping attacks. Always use AEAD modes like AES-GCM or ChaCha20-Poly1305.

```rust
// WRONG — no authentication
// let ciphertext = aes_cbc_encrypt(&key, &iv, plaintext);

// CORRECT — authenticated
// let ciphertext = aes_gcm_encrypt(&key, &nonce, plaintext, &associated_data);
```

### 4. Never Roll Your Own Crypto

Even if you understand the theory, implementing cryptographic primitives from scratch introduces subtle vulnerabilities. Use audited crates.

```rust
// WRONG — manual XOR stream cipher
fn bad_encrypt(key: &[u8], data: &[u8]) -> Vec<u8> {
    data.iter().zip(key.iter().cycle()).map(|(a, b)| a ^ b).collect()
}

// CORRECT — use ChaCha20Poly1305 or AES-GCM
```

### 5. Use OS-Provided Randomness for Secrets

```rust
use rand::rngs::OsRng;
use rand::RngCore;

fn generate_token() -> [u8; 32] {
    let mut token = [0u8; 32];
    OsRng.fill_bytes(&mut token);
    token
}

// WRONG — predictable seed
// use rand::SeedableRng;
// let mut rng = rand::rngs::StdRng::seed_from_u64(42);
```

### 6. Validate Nonce Uniqueness

Reusing a nonce with AES-GCM is catastrophic — it destroys both confidentiality and authenticity. Use a counter or randomly generated nonces (with a 96-bit nonce, random is safe for up to 2^32 messages).

```rust
// Safe: generate a fresh random nonce for each message
let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

// DANGEROUS: reusing a static nonce
// let nonce = Nonce::from_slice(b"fixed nonce 123"); // Never do this!
```

### 7. Audit Your Dependency Tree

Use `cargo audit` to check for known vulnerabilities:

```bash
cargo install cargo-audit
cargo audit
```

Use `cargo deny` for supply-chain policy enforcement:

```bash
cargo install cargo-deny
cargo deny check advisories
```

---

## 11. Summary Table

| Task | Recommended Crate | Notes |
|---|---|---|
| Hash map hashing | `std` (`ahash`) or `rustc-hash` | Non-cryptographic |
| File integrity / checksums | `sha2`, `sha3`, `blake3` | Use BLAKE3 for new code |
| Message authentication | `hmac` + `sha2`, or `blake3` keyed | HMAC-SHA256 is the standard |
| Password storage | `argon2` | Never use SHA directly |
| Symmetric encryption | `aes-gcm`, `chacha20poly1305` | Always AEAD |
| Digital signatures | `ed25519-dalek`, `ring` | Ed25519 preferred |
| Key exchange | `ring` (X25519) | Use ephemeral keys |
| TLS | `rustls` | Avoid `openssl` in new code |
| Constant-time ops | `subtle` | For all secret comparisons |
| Memory zeroing | `zeroize` | For all key material |

---

## Further Reading

- [RustCrypto GitHub Organization](https://github.com/RustCrypto)
- [`ring` Documentation](https://docs.rs/ring)
- [Rust Cryptography Guidelines (ANSSI)](https://www.ssi.gouv.fr/en/guide/rust-security-guidelines/)
- [BLAKE3 paper](https://github.com/BLAKE3-team/BLAKE3-specs)
- [Argon2 RFC 9106](https://www.rfc-editor.org/rfc/rfc9106)