/*!
WARNING: This signature software is a prototype. It has been replaced by the final system
[Ed25519](http://ed25519.cr.yp.to/). It is only kept here for compatibility reasons.
*/
use libc::{c_ulonglong, c_int};
use std::slice::from_elem;

#[link(name = "sodium")]
extern {
    fn crypto_sign_edwards25519sha512batch_keypair(pk: *mut u8,
                                                   sk: *mut u8) -> c_int;
    fn crypto_sign_edwards25519sha512batch(sm: *mut u8,
                                           smlen: *mut c_ulonglong,
                                           m: *u8,
                                           mlen: c_ulonglong,
                                           sk: *u8) -> c_int;
    fn crypto_sign_edwards25519sha512batch_open(m: *mut u8,
                                                mlen: *mut c_ulonglong,
                                                sm: *u8,
                                                smlen: c_ulonglong,
                                                pk: *u8) -> c_int;
}

pub static SECRETKEYBYTES: uint = 64;
pub static PUBLICKEYBYTES: uint = 32;
pub static SIGNATUREBYTES: uint = 64;

/**
 * `SecretKey` for signatures
 *
 * When a `SecretKey` goes out of scope its contents
 * will be zeroed out
 */
pub struct SecretKey([u8, ..SECRETKEYBYTES]);
impl Drop for SecretKey {
    fn drop(&mut self) {
        let &SecretKey(ref mut buf) = self;
        for e in buf.mut_iter() { *e = 0 }
    }
}
/**
 * `PublicKey` for signatures
 */
pub struct PublicKey([u8, ..PUBLICKEYBYTES]);

/**
 * `gen_keypair()` randomly generates a secret key and a corresponding public
 * key.
 *
 * THREAD SAFETY: `gen_keypair()` is thread-safe provided that you have
 * called `sodiumoxide::init()` once before using any other function
 * from sodiumoxide.
 */
pub fn gen_keypair() -> (PublicKey, SecretKey) {
    unsafe {
        let mut pk = [0u8, ..PUBLICKEYBYTES];
        let mut sk = [0u8, ..SECRETKEYBYTES];
        crypto_sign_edwards25519sha512batch_keypair(pk.as_mut_ptr(),
                                                    sk.as_mut_ptr());
        (PublicKey(pk), SecretKey(sk))
    }
}

/**
 * `sign()` signs a message `m` using the signer's secret key `sk`.
 * `sign()` returns the resulting signed message `sm`.
 */
pub fn sign(m: &[u8],
            &SecretKey(sk): &SecretKey) -> ~[u8] {
    unsafe {
        let mut sm = from_elem(m.len() + SIGNATUREBYTES, 0u8);
        let mut smlen = 0;
        crypto_sign_edwards25519sha512batch(sm.as_mut_ptr(),
                                            &mut smlen,
                                            m.as_ptr(),
                                            m.len() as c_ulonglong,
                                            sk.as_ptr());
        sm.truncate(smlen as uint);
        sm
    }
}

/**
 * `verify()` verifies the signature in `sm` using the signer's public key `pk`.
 * `verify()` returns the message `Some(m)`.
 * If the signature fails verification, `verify()` returns `None`.
 */
pub fn verify(sm: &[u8],
              &PublicKey(pk): &PublicKey) -> Option<~[u8]> {
    unsafe {
        let mut m = from_elem(sm.len(), 0u8);
        let mut mlen = 0;
        if crypto_sign_edwards25519sha512batch_open(m.as_mut_ptr(),
                                                    &mut mlen,
                                                    sm.as_ptr(),
                                                    sm.len() as c_ulonglong,
                                                    pk.as_ptr()) == 0 {
            m.truncate(mlen as uint);
            Some(m)
        } else {
            None
        }
    }
}

#[test]
fn test_sign_verify() {
    use randombytes::randombytes;
    for i in range(0, 256u) {
        let (pk, sk) = gen_keypair();
        let m = randombytes(i);
        let sm = sign(m, &sk);
        let m2 = verify(sm, &pk);
        assert!(Some(m) == m2);
    }
}

#[test]
fn test_sign_verify_tamper() {
    use randombytes::randombytes;
    for i in range(0, 32u) {
        let (pk, sk) = gen_keypair();
        let m = randombytes(i);
        let mut sm = sign(m, &sk);
        for j in range(0, sm.len()) {
            sm[j] ^= 0x20;
            assert!(None == verify(sm, &pk));
            sm[j] ^= 0x20;
        }
    }
}
