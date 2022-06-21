// Taken from https://github.com/GabrielRPrada/pkce-rs/blob/master/src/lib.rs
// Works when targetting wasm

use rand::{thread_rng, Rng};
use sha2::{Digest, Sha256};

const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
    abcdefghijklmnopqrstuvwxyz\
    0123456789-.~_";

pub fn code_verifier(length: usize) -> Vec<u8> {
    assert!(
        (43..=128).contains(&length),
        "Code verifier length must be between 43 and 128 bytes"
    );

    let mut rng = thread_rng();

    (0..length)
        .map(|_| {
            let i = rng.gen_range(0..CHARS.len());
            CHARS[i]
        })
        .collect()
}

fn base64_url_encode(input: &[u8]) -> String {
    let b64 = base64::encode(input);
    b64.chars()
        .filter_map(|c| match c {
            '=' => None,
            '+' => Some('-'),
            '/' => Some('_'),
            x => Some(x),
        })
        .collect()
}

pub fn code_challenge(code_verifier: &[u8]) -> String {
    let mut sha = Sha256::new();
    sha.update(code_verifier);
    let result = sha.finalize();
    base64_url_encode(&result[..])
}
