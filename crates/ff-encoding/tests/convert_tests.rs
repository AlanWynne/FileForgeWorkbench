//! Integration tests for encoding conversion.

use ff_encoding::*;

#[test]
fn roundtrip_ascii_through_iso_8859_1() {
    // Validates: Requirement 3.1, 4.1
    let registry = EncodingRegistry::new();
    let encoding = registry.by_name("iso-8859-1").unwrap();
    let text = "Hello, world! 123";

    let encoded = convert_from_utf8(text, encoding, UnmappableAction::Abort).unwrap();
    let decoded = convert_to_utf8(&encoded.data, encoding).unwrap();
    assert_eq!(String::from_utf8(decoded.data).unwrap(), text);
}

#[test]
fn roundtrip_utf16le() {
    // Validates: Requirement 3.2, 4.6
    let registry = EncodingRegistry::new();
    let encoding = registry.by_name("utf-16le").unwrap();
    let text = "Hello, 世界! 🎉";

    let encoded = convert_from_utf8(text, encoding, UnmappableAction::Abort).unwrap();
    let decoded = convert_to_utf8(&encoded.data, encoding).unwrap();
    assert_eq!(String::from_utf8(decoded.data).unwrap(), text);
}

#[test]
fn roundtrip_utf16be() {
    // Validates: Requirement 3.2, 4.6
    let registry = EncodingRegistry::new();
    let encoding = registry.by_name("utf-16be").unwrap();
    let text = "Hello, 世界!";

    let encoded = convert_from_utf8(text, encoding, UnmappableAction::Abort).unwrap();
    let decoded = convert_to_utf8(&encoded.data, encoding).unwrap();
    assert_eq!(String::from_utf8(decoded.data).unwrap(), text);
}

#[test]
fn roundtrip_utf32le() {
    // Validates: Requirement 3.2, 4.6
    let registry = EncodingRegistry::new();
    let encoding = registry.by_name("utf-32le").unwrap();
    let text = "Hello 😀";

    let encoded = convert_from_utf8(text, encoding, UnmappableAction::Abort).unwrap();
    let decoded = convert_to_utf8(&encoded.data, encoding).unwrap();
    assert_eq!(String::from_utf8(decoded.data).unwrap(), text);
}

#[test]
fn unmappable_character_abort() {
    // Validates: Requirement 4.4, 4.5
    let registry = EncodingRegistry::new();
    let encoding = registry.by_name("iso-8859-1").unwrap();
    let text = "Hello 😀"; // Emoji not in ISO-8859-1

    let result = convert_from_utf8(text, encoding, UnmappableAction::Abort);
    assert!(result.is_err());
}

#[test]
fn unmappable_character_replace() {
    // Validates: Requirement 4.5
    let registry = EncodingRegistry::new();
    let encoding = registry.by_name("iso-8859-1").unwrap();
    let text = "Hi\u{0100}!"; // Ā is U+0100, not in ISO-8859-1

    let result = convert_from_utf8(
        text,
        encoding,
        UnmappableAction::ReplaceWithPlaceholder('?'),
    )
    .unwrap();
    assert_eq!(result.data, b"Hi?!");
    assert_eq!(result.issues.len(), 1);
}

#[test]
fn streaming_decoder_split_multibyte() {
    // Validates: Requirement 3.8
    let registry = EncodingRegistry::new();
    let encoding = registry.by_name("utf-8").unwrap();

    let text = "Héllo"; // H(1) é(2) l(1) l(1) o(1) = 6 bytes
    let bytes = text.as_bytes();

    let mut decoder = StreamDecoder::new(encoding);

    // Split in the middle of 'é' (bytes 1-2 are C3 A9)
    let r1 = decoder.decode_chunk(&bytes[..2]).unwrap();
    let r2 = decoder.decode_chunk(&bytes[2..]).unwrap();
    let final_r = decoder.finish().unwrap();

    let mut full_result = String::from_utf8(r1.data).unwrap();
    full_result.push_str(&String::from_utf8(r2.data).unwrap());
    full_result.push_str(&String::from_utf8(final_r.data).unwrap());
    assert_eq!(full_result, text);
}

#[test]
fn streaming_encoder_produces_valid_output() {
    // Validates: Requirement 4.8
    let registry = EncodingRegistry::new();
    let encoding = registry.by_name("utf-16le").unwrap();

    let mut encoder = StreamEncoder::new(encoding, UnmappableAction::Abort);
    let r1 = encoder.encode_chunk("Hello").unwrap();
    let r2 = encoder.encode_chunk(" World").unwrap();
    let r_final = encoder.finish().unwrap();

    let mut full_bytes = r1.data;
    full_bytes.extend_from_slice(&r2.data);
    full_bytes.extend_from_slice(&r_final.data);

    // Decode back
    let decoded = convert_to_utf8(&full_bytes, encoding).unwrap();
    assert_eq!(String::from_utf8(decoded.data).unwrap(), "Hello World");
}
