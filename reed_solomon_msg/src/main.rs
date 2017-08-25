extern crate reed_solomon;

use std::str;
use reed_solomon::Encoder;
use reed_solomon::Decoder;

fn main() {
    let data = b"aabbaabbaabbaabbaabbaabbaabbaabbaabb";

    let ecc_len = 14;

    let enc = Encoder::new(ecc_len);
    let dec = Decoder::new(ecc_len);

    let encoded = enc.encode(&data[..]);

    println!("Encoded = {:?}", encoded);

    let mut corrupted = *encoded;
    for i in 0 .. ecc_len / 2 {
        corrupted[i] = 0x1;
    }

    let recovered = dec.correct(&mut corrupted, None).unwrap();
    let orig_str = str::from_utf8(data).unwrap();
    let recv_str = str::from_utf8(recovered.data()).unwrap();

    println!("message:               {:?}", orig_str);
    // println!("original data:         {:?}", data);
    println!("error correction code: {:?}", encoded.ecc());
    println!("corrupted:             {:?}", corrupted);
    println!("repaired:              {:?}", recv_str);

}
