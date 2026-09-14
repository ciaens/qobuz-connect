use std::io::Read as _;

use qobuz_connect::wire::{self, Frame};

fn main() {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("expected hex on stdin");
        return;
    }
    match wire::decode(&hex(&input)) {
        Ok(frames) => {
            for frame in frames {
                println!("{frame:#?}");
                if let Frame::Payload(payload) = frame {
                    println!("{:#?}", wire::messages(&payload));
                }
            }
        }
        Err(err) => eprintln!("{err}"),
    }
}

fn hex(text: &str) -> Vec<u8> {
    let digits: Vec<u8> = text.bytes().filter(u8::is_ascii_hexdigit).collect();
    digits
        .chunks(2)
        .filter_map(|pair| std::str::from_utf8(pair).ok())
        .filter_map(|pair| u8::from_str_radix(pair, 16).ok())
        .collect()
}
