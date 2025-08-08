use solana_sdk::{signature::Keypair, signer::Signer};

use super::keys::*;

fn create_metadata() -> MetadataV1 {
    MetadataV1 {
        created_at: 1754401735372,
        valid_for: 5_000_000_000,
        usage_limit: 1234,
        scopes: vec![Scope::CompletionModel],
    }
}

fn create_sk() -> SecretKeyV1 {
    let keypair = Keypair::new();
    let metadata = create_metadata();
    let bytes_to_sign = MetadataRawV1::try_from(metadata.clone())
        .unwrap()
        .into_bytes();
    let signature = keypair.sign_message(&bytes_to_sign[..]).to_string();

    // FAKE SIGNATURE
    SecretKeyV1 {
        version: 1,
        wallet: Wallet::Solana,
        signer: keypair.pubkey().to_string(),
        signature,
        metadata: create_metadata(),
    }
}

fn sk_json() -> String {
    // FAKE SIGNER AND SIGNATURE
    serde_json::json!({
        "version": 1,
        "wallet": "solana",
        "signer": "8W7X1tGnWh9CXwnPD7wgke31Gdcqmex4LapJvQ2afBUq",
        "signature": "3HErCXKpy76bbu2rr1BpV79ue2N1StxaPwd4qRjQERMsY15JCpg4gDsN9jQ8cDNkmjeFxkc1GSEHzKULJA8mH6qL",
        "metadata": {
            "created_at": 1754401735372u64,
            "valid_for": 5000000000u64,
            "usage_limit": 1234,
            "scopes": ["completion_model"]
        }
    })
    .to_string()
}

#[test]
fn test_sk_encode_decode() {
    let sk = create_sk();
    let sk_encoded = sk.clone().into_string("test").unwrap();
    println!("{sk_encoded}");
    let (scope, sk_decoded) = SecretKeyV1::decode(&sk_encoded).unwrap();

    assert_eq!(scope, "test");
    assert_eq!(sk_decoded.signer, sk.signer);
    assert_eq!(sk_decoded.signature, sk.signature);
    assert_eq!(sk_decoded.metadata.created_at, sk.metadata.created_at);
}

#[test]
fn test_sk_parse() {
    let sk_json_str = sk_json();
    // println!("Parsing deserializing secret key from:\n{sk_json_str}");
    let sk_parsed = serde_json::from_str::<SecretKeyV1>(&sk_json_str).unwrap();

    assert_eq!(sk_parsed.metadata.scopes.len(), 1);
    assert_eq!(sk_parsed.metadata.scopes[0], Scope::CompletionModel);
}

#[test]
fn test_raw_key() {
    let sk = create_sk();
    let raw = SecretKeyRawV1::try_from(sk).unwrap();
    let bytes = raw.into_bytes();

    assert_eq!(bytes.len(), SecretKeyRawV1::BYTES);
}

#[test]
fn test_raw_metadata() {
    let sk = create_sk();
    let raw = MetadataRawV1::try_from(sk.metadata).unwrap();
    let bytes = raw.into_bytes();

    assert_eq!(bytes.len(), MetadataRawV1::BYTES);
}

#[test]
fn test_verify() {
    let sk = create_sk();
    assert!(sk.verify_signature().is_ok());

    // encode, decode and verify
    let sk = create_sk();
    let sk_string = sk.into_string("test").unwrap();
    let (_, sk) = SecretKeyV1::decode(&sk_string).unwrap();
    assert!(sk.verify_signature().is_ok());
}
