use sgx_tcrypto::SgxEccHandle;
use sgx_types::*;

pub struct PrivateKey {
    d: sgx_ec256_private_t,
    public_key: PublicKey,
}

pub struct PublicKey {
    pub xy: sgx_ec256_public_t,

    curve: SgxEccHandle,
}

impl PrivateKey {
    pub fn as_bytes(&self) -> [u8; 32] {
        let mut d = self.d.r.clone();
        d.reverse();
        d
    }

    pub fn public(&self) -> &PublicKey {
        &self.public_key
    }
}

impl PublicKey {
    pub fn marshal_as_uncompressed(&self) -> [u8; 65] {
        let mut out = [0u8; 65];

        // uncompressed tag
        out[0] = 0x04;

        out[1..33].copy_from_slice(&self.xy.gx);
        out[1..33].reverse();

        out[33..].copy_from_slice(&self.xy.gy);
        out[33..].reverse();

        out
    }
}

pub fn generate_key() -> Result<PrivateKey, sgx_status_t> {
    let curve = SgxEccHandle::new();

    curve.open()?;

    let (priv_key, pub_key) = curve.create_key_pair()?;

    let private_key = PrivateKey {
        d: priv_key,
        public_key: PublicKey { xy: pub_key, curve },
    };

    // close would be called by drop automatically
    //ecc_handle.close();

    Ok(private_key)
}

/// sign the given with SHA256
pub fn sign(priv_key: &PrivateKey, msg: &[u8]) -> Result<sgx_ec256_signature_t, sgx_status_t> {
    priv_key.public_key.curve.ecdsa_sign_slice(msg, &priv_key.d)
}
