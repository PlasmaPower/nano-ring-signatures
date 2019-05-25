#![allow(non_snake_case)] // for r vs R
//! https://paper.dropbox.com/doc/Ring-Signatures-Part-2-AOS-Rings-Zf8c3AL0mg9bPp05PeZl6
use blake2::{Blake2b, Digest};
use curve25519_dalek::constants::ED25519_BASEPOINT_TABLE;
use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::scalar::Scalar;
use nanocurrency_types::Account;
use rand::rngs::OsRng;
use std::env;
use std::io::{self, prelude::*};

fn hram_e_value(prev_R_value: &[u8], pk: &[u8], msg: &[u8]) -> Scalar {
    // e = hram = H(R || A || M)
    let mut hasher = Blake2b::default();
    hasher.input(&prev_R_value);
    hasher.input(pk);
    hasher.input(msg);
    Scalar::from_hash(hasher)
}

fn get_next_e_value(
    msg: &[u8],
    pk: &EdwardsPoint,
    prev_e_value: &Scalar,
    s_value: &Scalar,
    next_pk_bytes: &[u8],
) -> Scalar {
    let prev_R_value = s_value * &ED25519_BASEPOINT_TABLE - prev_e_value * pk;
    let prev_R_value_bytes = prev_R_value.compress().to_bytes();
    hram_e_value(&prev_R_value_bytes, next_pk_bytes, msg)
}

fn expand_sk(sk: &[u8]) -> ([u8; 32], [u8; 32]) {
    let mut hasher: Blake2b = Blake2b::default();
    hasher.input(sk);
    let hash = hasher.result();

    let mut scalar: [u8; 32] = [0u8; 32];
    let mut r_material: [u8; 32] = [0u8; 32];
    scalar.copy_from_slice(&hash.as_slice()[..32]);
    r_material.copy_from_slice(&hash.as_slice()[32..]);
    scalar[0] &= 248;
    scalar[31] &= 63;
    scalar[31] |= 64;

    (scalar, r_material)
}

fn main() {
    let mut args = env::args();
    args.next();
    let generate = match args.next().as_ref().map(|s| s.as_str()) {
        Some("generate") => true,
        Some("verify") | Some("validate") => false,
        _ => panic!("Expected either \"generate\" or \"verify\" as argument"),
    };
    let mut reading_str = String::new();
    if generate {
        let sk_str = rpassword::read_password_from_tty(Some("Private key: "))
            .expect("Failed to read private key from TTY");
        let sk = hex::decode(sk_str).expect("Failed to decode private key as hex");
        if sk.len() != 32 {
            panic!("Private key is {} bytes long, expected 32 bytes", sk.len());
        }
        let (sk_scalar_bytes, _) = expand_sk(&sk);
        let sk_scalar = Scalar::from_bytes_mod_order(sk_scalar_bytes);
        let our_pk = &sk_scalar * &ED25519_BASEPOINT_TABLE;
        let our_pk_bytes = our_pk.compress().to_bytes();
        let mut pks = Vec::new();
        pks.push((our_pk_bytes, our_pk));
        loop {
            print!("Account or empty to finish: ");
            io::stdout().flush().expect("Failed to flush stdout");
            reading_str.clear();
            io::stdin()
                .read_line(&mut reading_str)
                .expect("Failed to read form stdin");
            let reading_str = reading_str.trim();
            if reading_str.is_empty() {
                break;
            }
            // Address
            let account: Account = reading_str
                .parse()
                .expect("Failed to parse input as account");
            let curve_point = CompressedEdwardsY(account.0.clone())
                .decompress()
                .expect("Failed to decompress account as compressed curve point");
            pks.push((account.0, curve_point));
        }
        pks.sort_by(|(b1, _), (b2, _)| b1.cmp(&b2));
        pks.dedup_by(|(_, p1), (_, p2)| p1 == p2);
        let our_i = pks
            .iter()
            .position(|(b, _)| b == &our_pk_bytes)
            .expect("Failed to find own public key");
        print!("Message: ");
        io::stdout().flush().expect("Failed to flush stdout");
        reading_str.clear();
        reading_str.extend("ring signature".chars());
        io::stdin()
            .read_line(&mut reading_str)
            .expect("Failed to read form stdin");
        if reading_str.ends_with('\n') {
            reading_str.pop();
        }
        if reading_str.ends_with('\r') {
            reading_str.pop();
        }
        let message = reading_str.into_bytes();
        let mut rng = OsRng::new().expect("Failed to get OS RNG");
        let our_r_value = Scalar::random(&mut rng);
        let our_R_value_bytes = (&our_r_value * &ED25519_BASEPOINT_TABLE)
            .compress()
            .to_bytes();
        let mut next_i = (our_i + 1) % pks.len();
        let mut first_e_value = None;
        let mut next_e_value = hram_e_value(&our_R_value_bytes, &pks[next_i].0, &message);
        if next_i == 0 {
            first_e_value = Some(next_e_value.clone());
        }
        let mut s_values = vec![None; pks.len()];
        while next_i != our_i {
            let pk = &pks[next_i];
            let s_value = Scalar::random(&mut rng);
            s_values[next_i] = Some(s_value);
            next_i = (next_i + 1) % pks.len();
            next_e_value =
                get_next_e_value(&message, &pk.1, &next_e_value, &s_value, &pks[next_i].0);
            if next_i == 0 {
                first_e_value = Some(next_e_value.clone());
            }
        }
        s_values[our_i] = Some(our_r_value + next_e_value * sk_scalar);
        let first_e_value = first_e_value.expect("Didn't generate first e value");
        println!();
        println!("Sorted accounts list:");
        for (pk_bytes, _) in pks {
            println!("{}", Account(pk_bytes));
        }
        let mut signature = first_e_value.to_bytes().to_vec();
        for s_value in s_values {
            let s_value = s_value.expect("Didn't generate s value");
            signature.extend(&s_value.to_bytes());
        }
        println!("\nSignature:\n{}", hex::encode(signature));
    } else {
        let mut pks = Vec::new();
        let signature;
        loop {
            print!("Account or signature to finish: ");
            io::stdout().flush().expect("Failed to flush stdout");
            reading_str.clear();
            io::stdin()
                .read_line(&mut reading_str)
                .expect("Failed to read form stdin");
            let reading_str = reading_str.trim();
            if reading_str.contains('_') {
                // Address
                let account: Account = reading_str
                    .parse()
                    .expect("Failed to parse input as account");
                let curve_point = CompressedEdwardsY(account.0.clone())
                    .decompress()
                    .expect("Failed to decompress account as compressed curve point");
                pks.push((account.0, curve_point));
            } else {
                signature =
                    hex::decode(reading_str).expect("Failed to parse input as hex signature");
                break;
            }
        }
        pks.sort_by(|(b1, _), (b2, _)| b1.cmp(&b2));
        pks.dedup_by(|(_, p1), (_, p2)| p1 == p2);
        if pks.is_empty() {
            panic!("At least one account must be specified");
        }
        print!("Message: ");
        io::stdout().flush().expect("Failed to flush stdout");
        reading_str.clear();
        reading_str.extend("ring signature".chars());
        io::stdin()
            .read_line(&mut reading_str)
            .expect("Failed to read form stdin");
        if reading_str.ends_with('\n') {
            reading_str.pop();
        }
        if reading_str.ends_with('\r') {
            reading_str.pop();
        }
        let message = reading_str.into_bytes();
        let expected_len = 32 + 32 * pks.len();
        if signature.len() != expected_len {
            panic!(
                "Expected signature to be {} bytes long given number of accounts, got {} bytes",
                expected_len,
                signature.len(),
            );
        }
        let mut first_e_value_bytes = [0u8; 32];
        first_e_value_bytes.copy_from_slice(&signature[..32]);
        let first_e_value = Scalar::from_bytes_mod_order(first_e_value_bytes);
        let s_values = signature[32..]
            .chunks(32)
            .map(|chunk| {
                let mut bytes = [0u8; 32];
                bytes.copy_from_slice(chunk);
                Scalar::from_bytes_mod_order(bytes)
            })
            .collect::<Vec<_>>();
        assert_eq!(pks.len(), s_values.len());
        let mut pks_iter = pks.iter().cycle().peekable();
        let mut next_e_value = first_e_value.clone();
        for s_value in s_values.into_iter() {
            let pk = pks_iter.next().unwrap();
            let next_pk = pks_iter.peek().unwrap();
            next_e_value = get_next_e_value(&message, &pk.1, &next_e_value, &s_value, &next_pk.0);
        }
        if next_e_value == first_e_value {
            println!("Ring signature is valid! :)")
        } else {
            println!("Ring signature is invalid :(");
        }
    }
}
