use tfhe::prelude::*;
use tfhe::{
    generate_keys, set_server_key, ClientKey, CompressedServerKey, ConfigBuilder, FheUint8,
};

fn main() {
    let config = ConfigBuilder::all_disabled()
        .enable_default_integers()
        .build();

    let cks = ClientKey::generate(config);
    let compressed_sks = CompressedServerKey::new(&cks);

    println!(
        "compressed size  : {}",
        bincode::serialize(&compressed_sks).unwrap().len()
    );

    let sks = compressed_sks.decompress();

    println!(
        "decompressed size: {}",
        bincode::serialize(&sks).unwrap().len()
    );

    set_server_key(sks);

    let clear_a = 12u8;
    let a = FheUint8::try_encrypt(clear_a, &cks).unwrap();

    let c = a + 234u8;
    let decrypted: u8 = c.decrypt(&cks);
    assert_eq!(decrypted, clear_a.wrapping_add(234));
}
