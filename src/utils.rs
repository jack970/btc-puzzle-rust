use base58::FromBase58;
use num_bigint::BigInt;
use num_traits::Num;
use ripemd::{Digest, Ripemd160};
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

#[derive(Serialize, Deserialize)]
pub struct Range {
    pub min: String,
    pub max: String,
    pub status: i8,
}

pub fn carregar_json<T: DeserializeOwned>(filename: &str) -> T {
    let json_path = Path::new(filename);
    let mut r_data = String::new();
    let mut r_file = File::open(json_path).expect("Arquivo não encontrado!");

    r_file
        .read_to_string(&mut r_data)
        .expect("Erro ao ler arquivo json!");

    let data: T = serde_json::from_str(&r_data).unwrap();
    data
}

pub fn fazer_pergunta() -> String {
    let mut pergunta = String::new();
    let _ = io::stdout().flush();

    io::stdin()
        .read_line(&mut pergunta)
        .expect("Ocorreu um erro!");

    pergunta.trim().to_string()
}

pub fn escolher_carteira() -> (BigInt, BigInt, Vec<u8>) {
    let wallets_json: Vec<String> = carregar_json("src/data/wallets.json");
    let ranges_json: Vec<Range> = carregar_json("src/data/ranges.json");

    let carteira: usize = loop {
        print!("Insira uma Carteira [1 | {}]: ", ranges_json.len());
        let input = fazer_pergunta();

        match input.parse::<u32>() {
            Ok(n) if n >= 1 && n <= ranges_json.len() as u32 => break n as usize,
            Ok(_) => println!("Por favor, insira um número entre 1 e 161."),
            Err(_) => println!("Por favor, insira um número válido."),
        }
    };

    let wallet = converter_carteira_ripdem160(&wallets_json[carteira - 1]);

    let min = &ranges_json[carteira - 1].min;
    let max = &ranges_json[carteira - 1].max;

    let min_bigint = convert_bigint(min);
    let max_bigint = convert_bigint(max);

    (min_bigint, max_bigint, wallet)
}

fn converter_carteira_ripdem160(address: &str) -> Vec<u8> {
    let decoded = address
        .from_base58()
        .expect("Ocorreu um error em converter base58");

    let hash160 = &decoded[1..21];

    hash160.to_vec()
}

pub fn convert_bigint(value: &String) -> BigInt {
    let value_str = &value[2..];
    let value_bigint = BigInt::from_str_radix(value_str, 16).expect("Error to convert to bigint!");
    value_bigint
}

pub fn criar_chave_publica160(private_key: BigInt) -> Vec<u8> {
    let private_key_hex = format!("{:0>64x}", private_key);
    let private_key_bytes = hex::decode(private_key_hex).unwrap();

    // Create a publicKey compressed from the private key bytes
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&private_key_bytes).expect("Error converter secret key");
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    calcular_ripemd160(public_key)
}

pub fn calcular_ripemd160(public_key: PublicKey) -> Vec<u8> {
    let public_key_bytes = public_key.serialize();

    let sha256_hash = Sha256::digest(&public_key_bytes);

    let ripemd160 = Ripemd160::digest(&sha256_hash);
    ripemd160.to_vec()
    // let mut address_bytes = vec![0x00];
    // address_bytes.extend_from_slice(&ripemd160);

    // // Calcula o checksum (SHA-256 do SHA-256) e adiciona os primeiros 4 bytes ao final do endereço
    // let checksum = Sha256::digest(&Sha256::digest(&address_bytes));

    // address_bytes.extend_from_slice(&checksum[0..4]);

    // // Codifica o endereço em Base58Check
    // let address = address_bytes.to_base58();
    // address
}
