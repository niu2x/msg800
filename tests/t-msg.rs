use msg800::msg::Message;

#[test]
fn pack() {
    let mut msg = Message::new();
    msg.write_bytes(b"hello world");
    let block = msg.pack();
    println!("D {block:?}");
}
