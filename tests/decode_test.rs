use kmlcli::tiles::decode::zigzag_decode_pub;

#[test]
fn test_zigzag_decode() {
    assert_eq!(zigzag_decode_pub(0), 0);
    assert_eq!(zigzag_decode_pub(1), -1);
    assert_eq!(zigzag_decode_pub(2), 1);
    assert_eq!(zigzag_decode_pub(3), -2);
    assert_eq!(zigzag_decode_pub(4), 2);
}
