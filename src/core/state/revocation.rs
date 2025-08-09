pub struct Revocation {
    pub secret_key_hash: [u8; 32],
    pub revoked_at: i64,
}
