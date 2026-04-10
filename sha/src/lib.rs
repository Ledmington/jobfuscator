#![forbid(unsafe_code)]

pub fn sha256(input: &[u8]) -> [u8; 32] {
    do_sha256(&preprocess_input(input))
}

fn preprocess_input(input: &[u8]) -> Vec<u8> {
    let mut output = input.to_owned();

    let original_length_bits: u64 = (8 * output.len()).try_into().unwrap();

    // append the bit '1'
    output.push(0x80);

    // append '0' bits until the length of the input (in bytes) is 56 modulo 64
    while output.len() % 64 != 56 {
        output.push(0x00);
    }

    // append the original length of the message (in bits) as a 64 big-endian integer
    for x in original_length_bits.to_be_bytes() {
        output.push(x);
    }

    output
}

fn do_sha256(input: &[u8]) -> [u8; 32] {
    let mut hash: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let num_chunks = input.len() / 64;
    for chunk_index in 0..num_chunks {
        let mut w = [0u32; 64];
        for j in 0..16 {
            let base = chunk_index * 64 + j * 4;
            w[j] = u32::from_be_bytes([
                input[base],
                input[base + 1],
                input[base + 2],
                input[base + 3],
            ]);
        }

        for i in 16..64 {
            let s0: u32 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1: u32 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a: u32 = hash[0];
        let mut b: u32 = hash[1];
        let mut c: u32 = hash[2];
        let mut d: u32 = hash[3];
        let mut e: u32 = hash[4];
        let mut f: u32 = hash[5];
        let mut g: u32 = hash[6];
        let mut h: u32 = hash[7];

        for i in 0..64 {
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(k[i])
                .wrapping_add(w[i]);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        hash[0] = hash[0].wrapping_add(a);
        hash[1] = hash[1].wrapping_add(b);
        hash[2] = hash[2].wrapping_add(c);
        hash[3] = hash[3].wrapping_add(d);
        hash[4] = hash[4].wrapping_add(e);
        hash[5] = hash[5].wrapping_add(f);
        hash[6] = hash[6].wrapping_add(g);
        hash[7] = hash[7].wrapping_add(h);
    }

    let mut out = [0u8; 32];
    for (i, val) in hash.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&val.to_be_bytes());
    }

    out
}

#[cfg(test)]
mod tests {
    use rand::{Rng, RngExt, SeedableRng};
    use sha2::{Digest, Sha256};

    use super::*;

    #[test]
    fn known_hashes() {
        let cases = [
            (
                "",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                "abc",
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
            (
                "hello world",
                "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
            ),
            (
                "abcdefghijklmnopqrstuvwxyz",
                "71c480df93d6ae2f1efad1447c66c9525e316218cf51fc8d9ed832f2daf18b73",
            ),
            (
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
                "db4bfcbd4da0cd85a60c3c37d3fbd8805c77f15fc6b1fdfe614ee0a7c8fdb4c0",
            ),
            (
                "The quick brown fox jumps over the lazy dog",
                "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592",
            ),
        ];

        for (input, expected) in cases {
            let input_bytes = input.to_owned().into_bytes();
            let output = sha256(&input_bytes);
            let output_string: String = output.iter().map(|b| format!("{:02x}", b)).collect();
            assert_eq!(
                Sha256::digest(input_bytes)
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>(),
                expected
            );
            assert_eq!(
                expected, output_string,
                "input = '{input}'\n expected = {expected}\n actual   = {output_string}"
            );
        }
    }

    #[test]
    fn fuzzing() {
        let mut rnd = rand::rng();

        for _ in 0..1000 {
            let seed: u64 = rnd.next_u64();

            let mut seeded_rng = rand::rngs::SmallRng::seed_from_u64(seed);

            let length = seeded_rng.random_range(0..1024);
            let input_bytes: Vec<u8> = (0..length).map(|_| seeded_rng.random::<u8>()).collect();
            let input_bytes_hex: String = input_bytes
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();

            let output = sha256(&input_bytes);
            let actual: String = output.iter().map(|b| format!("{:02x}", b)).collect();

            let expected = Sha256::digest(&input_bytes)
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();

            assert_eq!(
                expected, actual,
                "Input: b'{input_bytes_hex}' (length={length}, seed=0x{seed:016x})\nExpected : {expected}\nActual   : {actual}"
            );
        }
    }
}
