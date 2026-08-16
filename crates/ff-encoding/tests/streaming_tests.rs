//! Integration tests for streaming encoder/decoder.

use ff_encoding::*;

#[test]
fn stream_decoder_utf8_complete_chunks() {
    // Validates: Requirement 3.8
    let registry = EncodingRegistry::new();
    let encoding = registry.by_name("utf-8").unwrap();
    let text = "Hello, World!";

    let mut decoder = StreamDecoder::new(encoding);
    let r1 = decoder.decode_chunk(b"Hello, ").unwrap();
    let r2 = decoder.decode_chunk(b"World!").unwrap();
    let r_final = decoder.finish().unwrap();

    let mut result = String::from_utf8(r1.data).unwrap();
    result.push_str(&String::from_utf8(r2.data).unwrap());
    result.push_str(&String::from_utf8(r_final.data).unwrap());
    assert_eq!(result, text);
}

#[test]
fn stream_decoder_utf8_split_at_multibyte() {
    // Validates: Requirement 3.8
    let registry = EncodingRegistry::new();
    let encoding = registry.by_name("utf-8").unwrap();

    // "世界" is E4 B8 96 E7 95 8C in UTF-8
    let text = "世界";
    let bytes = text.as_bytes();
    assert_eq!(bytes.len(), 6);

    let mut decoder = StreamDecoder::new(encoding);
    // Split in the middle of the first character
    let r1 = decoder.decode_chunk(&bytes[..1]).unwrap(); // Just E4
    let r2 = decoder.decode_chunk(&bytes[1..4]).unwrap(); // B8 96 + E7 (start of second)
    let r3 = decoder.decode_chunk(&bytes[4..]).unwrap(); // 95 8C
    let r_final = decoder.finish().unwrap();

    let mut result = String::from_utf8(r1.data).unwrap();
    result.push_str(&String::from_utf8(r2.data).unwrap());
    result.push_str(&String::from_utf8(r3.data).unwrap());
    result.push_str(&String::from_utf8(r_final.data).unwrap());
    assert_eq!(result, text);
}

#[test]
fn stream_encoder_utf16le_multi_chunk() {
    // Validates: Requirement 4.8
    let registry = EncodingRegistry::new();
    let encoding = registry.by_name("utf-16le").unwrap();

    let mut encoder = StreamEncoder::new(encoding, UnmappableAction::Abort);
    let r1 = encoder.encode_chunk("AB").unwrap();
    let r2 = encoder.encode_chunk("CD").unwrap();
    let r_final = encoder.finish().unwrap();

    let mut full = r1.data;
    full.extend_from_slice(&r2.data);
    full.extend_from_slice(&r_final.data);

    // Verify: each char is 2 bytes in UTF-16LE
    assert_eq!(full.len(), 8);
    assert_eq!(&full[0..2], &[b'A', 0]);
    assert_eq!(&full[2..4], &[b'B', 0]);
    assert_eq!(&full[4..6], &[b'C', 0]);
    assert_eq!(&full[6..8], &[b'D', 0]);
}

#[test]
fn stream_decoder_empty_chunks() {
    // Edge case: empty chunks should be handled gracefully
    let registry = EncodingRegistry::new();
    let encoding = registry.by_name("utf-8").unwrap();

    let mut decoder = StreamDecoder::new(encoding);
    let r1 = decoder.decode_chunk(b"").unwrap();
    let r2 = decoder.decode_chunk(b"Hi").unwrap();
    let r3 = decoder.decode_chunk(b"").unwrap();
    let r_final = decoder.finish().unwrap();

    let mut result = String::from_utf8(r1.data).unwrap();
    result.push_str(&String::from_utf8(r2.data).unwrap());
    result.push_str(&String::from_utf8(r3.data).unwrap());
    result.push_str(&String::from_utf8(r_final.data).unwrap());
    assert_eq!(result, "Hi");
}

#[test]
fn stream_encoder_finish_with_no_pending() {
    // Validates: Requirement 4.8
    let registry = EncodingRegistry::new();
    let encoding = registry.by_name("utf-8").unwrap();

    let encoder = StreamEncoder::new(encoding, UnmappableAction::Abort);
    let r = encoder.finish().unwrap();
    assert!(r.data.is_empty());
}
