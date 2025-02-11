//! Pure Rust implementation of the [`crypto_box`] public-key authenticated
//! encryption scheme from [NaCl]-family libraries (e.g. libsodium, TweetNaCl)
//! which combines the [X25519] Diffie-Hellman function and the
//! [XSalsa20Poly1305] authenticated encryption cipher into an Elliptic Curve
//! Integrated Encryption Scheme ([ECIES]).
//!
//! # Introduction
//!
//! Imagine Alice wants something valuable shipped to her. Because it's
//! valuable, she wants to make sure it arrives securely (i.e. hasn't been
//! opened or tampered with) and that it's not a forgery (i.e. it's actually
//! from the sender she's expecting it to be from and nobody's pulling the old
//! switcheroo).
//!
//! One way she can do this is by providing the sender (let's call him Bob)
//! with a high-security box of her choosing. She provides Bob with this box,
//! and something else: a padlock, but a padlock without a key. Alice is
//! keeping that key all to herself. Bob can put items in the box then put the
//! padlock onto it, but once the padlock snaps shut, the box cannot be opened
//! by anyone who doesn't have Alice's private key.
//!
//! Here's the twist though, Bob also puts a padlock onto the box. This padlock
//! uses a key Bob has published to the world, such that if you have one of
//! Bob's keys, you know a box came from him because Bob's keys will open Bob's
//! padlocks (let's imagine a world where padlocks cannot be forged even if you
//! know the key). Bob then sends the box to Alice.
//!
//! In order for Alice to open the box, she needs two keys: her private key
//! that opens her own padlock, and Bob's well-known key. If Bob's key doesn't
//! open the second padlock then Alice knows that this is not the box she was
//! expecting from Bob, it's a forgery.
//!
//! # Usage
//!
//! NOTE: The following examples assume use of the `std`
//! [feature](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#choosing-features).
//!
//! ```rust
//! # #[cfg(feature = "std")]
//! # {
//! use crypto_box::{Box, PublicKey, SecretKey, aead::Aead};
//!
//! //
//! // Encryption
//! //
//!
//! // Generate a random secret key.
//! // NOTE: The secret key bytes can be accessed by calling `secret_key.as_bytes()`
//! let mut rng = crypto_box::rand_core::OsRng;
//! let alice_secret_key = SecretKey::generate(&mut rng);
//!
//! // Get the public key for the secret key we just generated
//! let alice_public_key_bytes = alice_secret_key.public_key().as_bytes().clone();
//!
//! // Obtain your recipient's public key.
//! let bob_public_key = PublicKey::from([
//!    0xe8, 0x98, 0xc, 0x86, 0xe0, 0x32, 0xf1, 0xeb,
//!    0x29, 0x75, 0x5, 0x2e, 0x8d, 0x65, 0xbd, 0xdd,
//!    0x15, 0xc3, 0xb5, 0x96, 0x41, 0x17, 0x4e, 0xc9,
//!    0x67, 0x8a, 0x53, 0x78, 0x9d, 0x92, 0xc7, 0x54,
//! ]);
//!
//! // Create a `Box` by performing Diffie-Hellman key agreement between
//! // the two keys.
//! let alice_box = Box::new(&bob_public_key, &alice_secret_key);
//!
//! // Get a random nonce to encrypt the message under
//! let nonce = crypto_box::generate_nonce(&mut rng);
//!
//! // Message to encrypt
//! let plaintext = b"Top secret message we're encrypting";
//!
//! // Encrypt the message using the box
//! let ciphertext = alice_box.encrypt(&nonce, &plaintext[..]).unwrap();
//!
//! //
//! // Decryption
//! //
//!
//! // Either side can encrypt or decrypt messages under the Diffie-Hellman key
//! // they agree upon. The example below shows Bob's side.
//! let bob_secret_key = SecretKey::from([
//!     0xb5, 0x81, 0xfb, 0x5a, 0xe1, 0x82, 0xa1, 0x6f,
//!     0x60, 0x3f, 0x39, 0x27, 0xd, 0x4e, 0x3b, 0x95,
//!     0xbc, 0x0, 0x83, 0x10, 0xb7, 0x27, 0xa1, 0x1d,
//!     0xd4, 0xe7, 0x84, 0xa0, 0x4, 0x4d, 0x46, 0x1b
//! ]);
//!
//! // Deserialize Alice's public key from bytes
//! let alice_public_key = PublicKey::from(alice_public_key_bytes);
//!
//! // Bob can compute the same Box as Alice by performing the reciprocal
//! // key exchange operation.
//! let bob_box = Box::new(&alice_public_key, &bob_secret_key);
//!
//! // Decrypt the message, using the same randomly generated nonce
//! let decrypted_plaintext = bob_box.decrypt(&nonce, &ciphertext[..]).unwrap();
//!
//! assert_eq!(&plaintext[..], &decrypted_plaintext[..]);
//! # }
//! ```
//!
//! ## Choosing `ChaChaBox` vs `SalasaBox`
//!
//! Currently, `crypto_box::Box` is default to use `xsalsa20poly1305` which doesn't support non-empty associated data
//! field. To specify customized AD, you can use `crypto_box::ChaChaBox` instead.
//!
//! ```rust
//! # #[cfg(feature = "std")]
//! # {
//! use crypto_box::{ChaChaBox, PublicKey, SecretKey, aead::{Aead, Payload}};
//!
//! let mut rng = crypto_box::rand_core::OsRng;
//! let alice_secret_key = SecretKey::generate(&mut rng);
//! let alice_public_key_bytes = alice_secret_key.public_key().as_bytes().clone();
//! let bob_public_key = PublicKey::from([
//!    0xe8, 0x98, 0xc, 0x86, 0xe0, 0x32, 0xf1, 0xeb,
//!    0x29, 0x75, 0x5, 0x2e, 0x8d, 0x65, 0xbd, 0xdd,
//!    0x15, 0xc3, 0xb5, 0x96, 0x41, 0x17, 0x4e, 0xc9,
//!    0x67, 0x8a, 0x53, 0x78, 0x9d, 0x92, 0xc7, 0x54,
//! ]);
//! let alice_box = ChaChaBox::new(&bob_public_key, &alice_secret_key);
//! let nonce = crypto_box::generate_nonce(&mut rng);
//!
//! // Message to encrypt
//! let plaintext = b"Top secret message we're encrypting".as_ref();
//! let associated_data = b"customized associated data here".as_ref();
//!
//! // Encrypt the message using the box
//! let ciphertext = alice_box.encrypt(&nonce, Payload {
//!   msg: plaintext, // your message to encrypt
//!   aad: associated_data, // not encrypted, but authenticated in tag
//! }).unwrap();
//!
//! //
//! // Decryption
//! //
//!
//! let bob_secret_key = SecretKey::from([
//!     0xb5, 0x81, 0xfb, 0x5a, 0xe1, 0x82, 0xa1, 0x6f,
//!     0x60, 0x3f, 0x39, 0x27, 0xd, 0x4e, 0x3b, 0x95,
//!     0xbc, 0x0, 0x83, 0x10, 0xb7, 0x27, 0xa1, 0x1d,
//!     0xd4, 0xe7, 0x84, 0xa0, 0x4, 0x4d, 0x46, 0x1b
//! ]);
//! let alice_public_key = PublicKey::from(alice_public_key_bytes);
//! let bob_box = ChaChaBox::new(&alice_public_key, &bob_secret_key);
//!
//! // Decrypt the message, using the same randomly generated nonce
//! let decrypted_plaintext = bob_box.decrypt(&nonce, Payload {
//!   msg: &ciphertext,
//!   aad: associated_data, // tag authentication will fail if associated data doesn't match, which fails the decryption
//! }).unwrap();
//!
//! assert_eq!(&plaintext[..], &decrypted_plaintext[..]);
//! }
//! ```
//!
//! ## In-place Usage (eliminates `alloc` requirement)
//!
//! This crate has an optional `alloc` feature which can be disabled in e.g.
//! microcontroller environments that don't have a heap.
//!
//! The [`AeadInPlace::encrypt_in_place`] and [`AeadInPlace::decrypt_in_place`]
//! methods accept any type that impls the [`aead::Buffer`] trait which
//! contains the plaintext for encryption or ciphertext for decryption.
//!
//! Note that if you enable the `heapless` feature of this crate,
//! you will receive an impl of `aead::Buffer` for [`heapless::Vec`]
//! (re-exported from the `aead` crate as `aead::heapless::Vec`),
//! which can then be passed as the `buffer` parameter to the in-place encrypt
//! and decrypt methods.
//!
//! A `heapless` usage example can be found in the documentation for the
//! `xsalsa20poly1305` crate:
//!
//! <https://docs.rs/xsalsa20poly1305/latest/xsalsa20poly1305/#in-place-usage-eliminates-alloc-requirement>
//!
//! [NaCl]: https://nacl.cr.yp.to/
//! [`crypto_box`]: https://nacl.cr.yp.to/box.html
//! [X25519]: https://cr.yp.to/ecdh.html
//! [XSalsa20Poly1305]: https://nacl.cr.yp.to/secretbox.html
//! [ECIES]: https://en.wikipedia.org/wiki/Integrated_Encryption_Scheme
//! [`heapless::Vec`]: https://docs.rs/heapless/latest/heapless/struct.Vec.html

#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_root_url = "https://docs.rs/crypto_box/0.7.1"
)]
#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use rand_core;
pub use xsalsa20poly1305::{aead, generate_nonce};

use chacha20::hchacha;
use chacha20poly1305::XChaCha20Poly1305;
use core::fmt::{self, Debug};
use rand_core::{CryptoRng, RngCore};
use salsa20::hsalsa20;
use x25519_dalek::{x25519, X25519_BASEPOINT_BYTES};
use xsalsa20poly1305::aead::{
    consts::{U0, U16, U24},
    generic_array::GenericArray,
    AeadCore, AeadInPlace, Buffer, Error, NewAead,
};
use xsalsa20poly1305::XSalsa20Poly1305;
use zeroize::{Zeroize, Zeroizing};

#[cfg(feature = "serde")]
use serde_crate::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

/// Size of a `crypto_box` public or secret key in bytes.
pub const KEY_SIZE: usize = 32;

/// Poly1305 tag.
///
/// Implemented as an alias for [`GenericArray`].
pub type Tag = GenericArray<u8, U16>;

/// A `crypto_box` secret key.
#[derive(Clone)]
pub struct SecretKey([u8; KEY_SIZE]);

impl SecretKey {
    /// Generate a random [`SecretKey`].
    pub fn generate<T>(csprng: &mut T) -> Self
    where
        T: RngCore + CryptoRng,
    {
        let mut bytes = [0u8; KEY_SIZE];
        csprng.fill_bytes(&mut bytes);
        SecretKey(bytes)
    }

    /// Get the [`PublicKey`] which corresponds to this [`SecretKey`]
    pub fn public_key(&self) -> PublicKey {
        PublicKey(x25519(self.0, X25519_BASEPOINT_BYTES))
    }

    #[deprecated(note = "use `as_bytes` instead")]
    #[allow(missing_docs)]
    pub fn to_bytes(&self) -> [u8; KEY_SIZE] {
        self.0
    }

    /// Get a slice of the [`SecretKey`] bytes
    pub fn as_bytes(&self) -> &[u8; KEY_SIZE] {
        &self.0
    }
}

impl From<[u8; KEY_SIZE]> for SecretKey {
    fn from(bytes: [u8; KEY_SIZE]) -> SecretKey {
        SecretKey(bytes)
    }
}

impl Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretKey(...)")
    }
}

impl Drop for SecretKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

/// A `crypto_box` public key.
///
/// This type can be serialized if the `serde` feature is enabled.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct PublicKey([u8; KEY_SIZE]);

impl PublicKey {
    /// Get a slice of the [`PublicKey`] bytes
    pub fn as_bytes(&self) -> &[u8; KEY_SIZE] {
        &self.0
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<&SecretKey> for PublicKey {
    fn from(secret_key: &SecretKey) -> PublicKey {
        secret_key.public_key()
    }
}

impl From<[u8; KEY_SIZE]> for PublicKey {
    fn from(bytes: [u8; KEY_SIZE]) -> PublicKey {
        PublicKey(bytes)
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use core::convert::TryInto;
        use serde_crate::de::{Error, SeqAccess, Visitor};

        struct PublicKeyVisitor;

        impl<'de> Visitor<'de> for PublicKeyVisitor {
            type Value = PublicKey;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a 32-byte public key")
            }

            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess<'de>,
            {
                let mut key_bytes = [0; KEY_SIZE];
                for i in 0..KEY_SIZE {
                    key_bytes[i] = match seq.next_element()? {
                        Some(val) => val,
                        None => {
                            return Err(Error::invalid_length(i - 1, &self));
                        }
                    }
                }
                Ok(PublicKey::from(key_bytes))
            }

            fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                // Convert to array (with length check)
                let array: [u8; KEY_SIZE] = bytes
                    .try_into()
                    .map_err(|_| Error::invalid_length(bytes.len(), &self))?;
                Ok(PublicKey::from(array))
            }
        }

        deserializer.deserialize_bytes(PublicKeyVisitor)
    }
}

macro_rules! impl_aead_in_place {
    ($box:ty, $nonce_size:ty, $tag_size:ty, $ct_overhead:ty) => {
        impl AeadCore for $box {
            type NonceSize = $nonce_size;
            type TagSize = $tag_size;
            type CiphertextOverhead = $ct_overhead;
        }

        impl AeadInPlace for $box {
            fn encrypt_in_place(
                &self,
                nonce: &GenericArray<u8, Self::NonceSize>,
                associated_data: &[u8],
                buffer: &mut dyn Buffer,
            ) -> Result<(), Error> {
                self.0.encrypt_in_place(nonce, associated_data, buffer)
            }

            fn encrypt_in_place_detached(
                &self,
                nonce: &GenericArray<u8, Self::NonceSize>,
                associated_data: &[u8],
                buffer: &mut [u8],
            ) -> Result<Tag, Error> {
                self.0
                    .encrypt_in_place_detached(nonce, associated_data, buffer)
            }

            fn decrypt_in_place(
                &self,
                nonce: &GenericArray<u8, Self::NonceSize>,
                associated_data: &[u8],
                buffer: &mut dyn Buffer,
            ) -> Result<(), Error> {
                self.0.decrypt_in_place(nonce, associated_data, buffer)
            }

            fn decrypt_in_place_detached(
                &self,
                nonce: &GenericArray<u8, Self::NonceSize>,
                associated_data: &[u8],
                buffer: &mut [u8],
                tag: &Tag,
            ) -> Result<(), Error> {
                self.0
                    .decrypt_in_place_detached(nonce, associated_data, buffer, tag)
            }
        }
    };
}

/// Alias for [`SalsaBox`].
pub type Box = SalsaBox;

/// Public-key encryption scheme based on the [X25519] Elliptic Curve
/// Diffie-Hellman function and the [XSalsa20Poly1305] authenticated encryption
/// cipher.
///
/// This type impls the [`aead::Aead`] trait, and otherwise functions as a
/// symmetric Authenticated Encryption with Associated Data (AEAD) cipher
/// once instantiated.
///
/// [X25519]: https://cr.yp.to/ecdh.html
/// [XSalsa20Poly1305]: https://github.com/RustCrypto/AEADs/tree/master/xsalsa20poly1305
#[derive(Clone)]
pub struct SalsaBox(XSalsa20Poly1305);

impl SalsaBox {
    /// Create a new [`SalsaBox`], performing X25519 Diffie-Hellman to derive
    /// a shared secret from the provided public and secret keys.
    pub fn new(public_key: &PublicKey, secret_key: &SecretKey) -> Self {
        let shared_secret = Zeroizing::new(x25519(secret_key.0, public_key.0));

        // Use HSalsa20 to create a uniformly random key from the shared secret
        let mut key = hsalsa20(
            GenericArray::from_slice(&*shared_secret),
            &GenericArray::default(),
        );

        let cipher = XSalsa20Poly1305::new(&key);
        key.zeroize();

        SalsaBox(cipher)
    }
}

impl_aead_in_place!(SalsaBox, U24, U16, U0);

/// Public-key encryption scheme based on the [X25519] Elliptic Curve
/// Diffie-Hellman function and the [XChaCha20Poly1305] authenticated encryption
/// cipher.
///
/// This type impls the [`aead::Aead`] trait, and otherwise functions as a
/// symmetric Authenticated Encryption with Associated Data (AEAD) cipher
/// once instantiated.
///
/// [X25519]: https://cr.yp.to/ecdh.html
/// [XChaCha20Poly1305]: https://github.com/RustCrypto/AEADs/blob/master/chacha20poly1305/
#[derive(Clone)]
pub struct ChaChaBox(XChaCha20Poly1305);

impl ChaChaBox {
    /// Create a new [`ChaChaBox`], performing X25519 Diffie-Hellman to derive
    /// a shared secret from the provided public and secret keys.
    pub fn new(public_key: &PublicKey, secret_key: &SecretKey) -> Self {
        let shared_secret = Zeroizing::new(x25519(secret_key.0, public_key.0));

        // Use HChaCha20 to create a uniformly random key from the shared secret
        let mut key = hchacha::<chacha20::R20>(
            GenericArray::from_slice(&*shared_secret),
            &GenericArray::default(),
        );

        let cipher = XChaCha20Poly1305::new(&key);
        key.zeroize();

        ChaChaBox(cipher)
    }
}

impl_aead_in_place!(ChaChaBox, U24, U16, U0);

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "serde")]
    fn test_public_key_serialization() {
        use super::PublicKey;
        use rand_core::RngCore;

        // Random PK bytes
        let mut public_key_bytes = [0; 32];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut public_key_bytes);

        // Create public key
        let public_key = PublicKey::from(public_key_bytes);

        // Round-trip serialize with bincode
        let serialized =
            bincode::serialize(&public_key).expect("Public key could not be serialized");
        let deserialized: PublicKey =
            bincode::deserialize(&serialized).expect("Public key could not be deserialized");
        assert_eq!(
            deserialized, public_key,
            "Deserialized public key does not match original"
        );

        // Round-trip serialize with rmp (msgpack)
        let serialized =
            rmp_serde::to_vec_named(&public_key).expect("Public key could not be serialized");
        let deserialized: PublicKey =
            rmp_serde::from_slice(&serialized).expect("Public key could not be deserialized");
        assert_eq!(
            deserialized, public_key,
            "Deserialized public key does not match original"
        );
    }
}
