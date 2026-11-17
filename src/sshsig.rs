//! The check of the signature of a release package. The owner signs the packages with `ssh-keygen -Y sign -n eh3d-release` (the same format and the same key as the
//! update helper of the server) and publishes the result next to the package as `<package>.sig`. This module reads that OpenSSH signature (`-----BEGIN SSH SIGNATURE-----`)
//! and checks it, with the public key of the releases that is built into the program, over the bytes of the package. Nothing here writes or reads anything else.

use std::io::Read;
use std::path::Path;

use ring::signature::{UnparsedPublicKey, ED25519};
use sha2::{Digest, Sha256, Sha512};

/// The public key of the releases, in the base64 of an OpenSSH public key (`ssh-ed25519 <this> ...`). Only the public half is here.
pub const RELEASE_KEY: &str =
    "AAAAC3NzaC1lZDI1NTE5AAAAILidTpyj1I0Eegn/A2tt+ATI9kbY74PhtPxKbQfszYHl";
/// The namespace the signatures are made in; a signature made for another purpose does not count.
pub const NAMESPACE: &str = "eh3d-release";
/// The most a signature may weigh: it is a few hundred bytes.
pub const MAX_SIGNATURE_BYTES: u64 = 4 * 1024;

fn base64_decode(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0u32);
    let mut padding = 0;
    for c in text.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => {
                padding += 1;
                continue;
            }
            b' ' | b'\t' | b'\r' | b'\n' => continue,
            _ => return None,
        };
        if padding > 0 {
            return None; // a digit after the padding
        }
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Some(out)
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        let part = self.data.get(self.pos..end)?;
        self.pos = end;
        Some(part)
    }

    fn u32(&mut self) -> Option<u32> {
        let b = self.take(4)?;
        Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// An SSH string: a length of four bytes and then that many bytes.
    fn string(&mut self) -> Option<&'a [u8]> {
        let n = self.u32()? as usize;
        self.take(n)
    }

    fn done(&self) -> bool {
        self.pos == self.data.len()
    }
}

fn push_string(out: &mut Vec<u8>, s: &[u8]) {
    out.extend_from_slice(&(s.len() as u32).to_be_bytes());
    out.extend_from_slice(s);
}

/// The binary of an armored signature.
fn unarmor(text: &str) -> Result<Vec<u8>, String> {
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    if lines.next() != Some("-----BEGIN SSH SIGNATURE-----") {
        return Err("not an OpenSSH signature".into());
    }
    let mut body = String::new();
    let mut closed = false;
    for line in lines {
        if line == "-----END SSH SIGNATURE-----" {
            closed = true;
            break;
        }
        body.push_str(line);
    }
    if !closed {
        return Err("the signature is cut short".into());
    }
    base64_decode(&body).ok_or_else(|| "the signature is not base64".to_string())
}

/// Hashes what is read with the algorithm the signature names.
fn digest(algorithm: &[u8], input: &mut dyn Read) -> Result<Vec<u8>, String> {
    fn run<D: Digest>(mut hasher: D, input: &mut dyn Read) -> Result<Vec<u8>, String> {
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            let n = input.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Ok(hasher.finalize().to_vec())
    }
    match algorithm {
        b"sha256" => run(Sha256::new(), input),
        b"sha512" => run(Sha512::new(), input),
        _ => Err("the signature uses a hash that is not accepted".into()),
    }
}

/// Checks that `signature` (the text of the `.sig` file) is a valid signature, made in the namespace of the releases by the key `key_b64`, of what `input` gives.
pub fn verify_with(signature: &str, key_b64: &str, input: &mut dyn Read) -> Result<(), String> {
    let trusted = base64_decode(key_b64).ok_or("the key of the releases is not base64")?;
    let mut key = Reader {
        data: &trusted,
        pos: 0,
    };
    if key.string() != Some(b"ssh-ed25519".as_slice()) {
        return Err("the key of the releases is not an Ed25519 key".into());
    }
    let public = key
        .string()
        .filter(|k| k.len() == 32)
        .ok_or("the key of the releases is wrong")?;

    let raw = unarmor(signature)?;
    let mut r = Reader { data: &raw, pos: 0 };
    if r.take(6) != Some(b"SSHSIG".as_slice()) || r.u32() != Some(1) {
        return Err("not an OpenSSH signature of version 1".into());
    }
    let signed_by = r.string().ok_or("the signature is cut short")?;
    if signed_by != trusted.as_slice() {
        return Err("the signature was not made with the key of the releases".into());
    }
    let namespace = r.string().ok_or("the signature is cut short")?;
    if namespace != NAMESPACE.as_bytes() {
        return Err("the signature was made for something else".into());
    }
    let reserved = r.string().ok_or("the signature is cut short")?;
    let algorithm = r.string().ok_or("the signature is cut short")?;
    let blob = r.string().ok_or("the signature is cut short")?;
    if !r.done() {
        return Err("the signature has more bytes than it should".into());
    }
    let mut inner = Reader { data: blob, pos: 0 };
    if inner.string() != Some(b"ssh-ed25519".as_slice()) {
        return Err("the signature is not an Ed25519 signature".into());
    }
    let sig = inner
        .string()
        .filter(|s| s.len() == 64)
        .ok_or("the signature is wrong")?;
    if !inner.done() {
        return Err("the signature has more bytes than it should".into());
    }

    let hashed = digest(algorithm, input)?;
    let mut message = b"SSHSIG".to_vec();
    push_string(&mut message, namespace);
    push_string(&mut message, reserved);
    push_string(&mut message, algorithm);
    push_string(&mut message, &hashed);
    UnparsedPublicKey::new(&ED25519, public)
        .verify(&message, sig)
        .map_err(|_| "the signature does not match the package".to_string())
}

/// Checks the signature of the package at `file` against the key of the releases.
pub fn verify_file(signature: &str, file: &Path) -> Result<(), String> {
    let mut f = std::fs::File::open(file).map_err(|e| e.to_string())?;
    verify_with(signature, RELEASE_KEY, &mut f)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Made by `ssh-keygen -Y sign -n eh3d-release` with a key that exists only for these tests, over the bytes of `PACKAGE`.
    const TEST_KEY: &str = "AAAAC3NzaC1lZDI1NTE5AAAAIKzHmEBAxZI/Ka1H9oEwqpgrEg7hnn8cYZ5lex16g/i9";
    const PACKAGE: &[u8] = b"EH3D test package\n";
    const SIG_SHA512: &str = "-----BEGIN SSH SIGNATURE-----
U1NIU0lHAAAAAQAAADMAAAALc3NoLWVkMjU1MTkAAAAgrMeYQEDFkj8prUf2gTCqmCsSDu
GefxxhnmV7HXqD+L0AAAAMZWgzZC1yZWxlYXNlAAAAAAAAAAZzaGE1MTIAAABTAAAAC3Nz
aC1lZDI1NTE5AAAAQG5dEl9fVQM27oM/I+yNF5MJJCsAx6irR57Sd3Qow+H/e9KB/Wkury
3gqbMj8P/tLA5ONgHEIIg+skkvnXBvrgA=
-----END SSH SIGNATURE-----
";
    const SIG_SHA256: &str = "-----BEGIN SSH SIGNATURE-----
U1NIU0lHAAAAAQAAADMAAAALc3NoLWVkMjU1MTkAAAAgrMeYQEDFkj8prUf2gTCqmCsSDu
GefxxhnmV7HXqD+L0AAAAMZWgzZC1yZWxlYXNlAAAAAAAAAAZzaGEyNTYAAABTAAAAC3Nz
aC1lZDI1NTE5AAAAQPdaGrzPqetWrley/9z2+YUUkFuvRO7cXdTXfYe/PqrXFdcgN4Tkjw
Ep0NaFuQWUFELcfIt8EBE8ZXt5sDFn/wQ=
-----END SSH SIGNATURE-----
";
    const SIG_OTHER_NAMESPACE: &str = "-----BEGIN SSH SIGNATURE-----
U1NIU0lHAAAAAQAAADMAAAALc3NoLWVkMjU1MTkAAAAgrMeYQEDFkj8prUf2gTCqmCsSDu
GefxxhnmV7HXqD+L0AAAAIb3RoZXItbnMAAAAAAAAABnNoYTUxMgAAAFMAAAALc3NoLWVk
MjU1MTkAAABAiB524j4zr0Yi1gpqtG3Jr0yJ/vZfrNzf/H/KtzlvxRMaIujvLC3Njx+Xqo
Atdlw0B0mca0q7nZ1eP2Qn58dGDw==
-----END SSH SIGNATURE-----
";

    fn check(sig: &str, package: &[u8]) -> Result<(), String> {
        let mut input = package;
        verify_with(sig, TEST_KEY, &mut input)
    }

    #[test]
    fn a_signature_made_by_ssh_keygen_verifies_with_either_hash() {
        assert_eq!(check(SIG_SHA512, PACKAGE), Ok(()));
        assert_eq!(check(SIG_SHA256, PACKAGE), Ok(()));
    }

    #[test]
    fn a_changed_package_does_not_verify() {
        assert!(check(SIG_SHA512, b"EH3D test package!\n").is_err());
        assert!(check(SIG_SHA512, b"").is_err());
    }

    #[test]
    fn a_signature_for_another_key_or_another_purpose_is_refused() {
        // the key of the releases is not the test key
        let mut input = PACKAGE;
        assert!(verify_with(SIG_SHA512, RELEASE_KEY, &mut input)
            .unwrap_err()
            .contains("key of the releases"));
        assert!(check(SIG_OTHER_NAMESPACE, PACKAGE)
            .unwrap_err()
            .contains("something else"));
    }

    #[test]
    fn a_signature_that_is_cut_or_changed_is_refused() {
        assert!(check("", PACKAGE).is_err());
        assert!(check("not a signature", PACKAGE).is_err());
        let cut = SIG_SHA512.replace("-----END SSH SIGNATURE-----", "");
        assert!(check(&cut, PACKAGE).is_err());
        // one letter of the signature itself changed
        let bad = SIG_SHA512.replace("3gqbMj8P", "3gqbMj8Q");
        assert_ne!(bad, SIG_SHA512);
        assert!(check(&bad, PACKAGE).is_err());
        // text after the signature is not part of it, but a body that is not base64 is refused
        let junk = SIG_SHA512.replace("U1NIU0lH", "U1NIU0l*");
        assert!(check(&junk, PACKAGE).is_err());
    }

    #[test]
    fn the_key_of_the_releases_is_a_well_formed_ed25519_key() {
        let key = base64_decode(RELEASE_KEY).unwrap();
        let mut r = Reader { data: &key, pos: 0 };
        assert_eq!(r.string(), Some(b"ssh-ed25519".as_slice()));
        assert_eq!(r.string().map(<[u8]>::len), Some(32));
        assert!(r.done());
    }

    #[test]
    fn the_base64_reader_handles_padding_and_refuses_other_letters() {
        assert_eq!(base64_decode("TWFu").unwrap(), b"Man");
        assert_eq!(base64_decode("TWE=").unwrap(), b"Ma");
        assert_eq!(base64_decode("TQ==").unwrap(), b"M");
        assert!(base64_decode("TW*u").is_none());
        assert!(base64_decode("TQ=A").is_none());
    }
}
