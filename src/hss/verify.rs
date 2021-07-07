use crate::{
    hasher::Hasher,
    lms::{self},
    LmotsParameter, LmsParameter,
};

use super::{definitions::HssPublicKey, signing::HssSignature};

pub fn verify<H: Hasher, const L: usize>(
    signature: &HssSignature<H, L>,
    public_key: &HssPublicKey<H, L>,
    message: &[u8],
) -> Result<(), &'static str> {
    if signature.level + 1 != public_key.level {
        return Err("Signature level and public key level does not match");
    }

    let mut key = &public_key.public_key;
    for i in 0..L - 1 {
        let sig = &signature.signed_public_keys[i].sig;
        let msg = &signature.signed_public_keys[i].public_key;

        if lms::verify::verify(sig, key, msg.to_binary_representation().as_slice()).is_err() {
            return Err("Could not verify next public key.");
        }
        key = msg;
    }

    lms::verify::verify(&signature.signature, key, message)
}

#[cfg(test)]
mod tests {
    use crate::hasher::sha256::Sha256Hasher;
    use crate::hasher::Hasher;
    use crate::hss::definitions::HssPrivateKey;
    use crate::hss::definitions::HssPublicKey;
    use crate::hss::signing::HssSignature;
    use crate::hss::verify::verify;
    use crate::LmotsAlgorithm;
    use crate::LmsAlgorithm;

    const LEVEL: usize = 2;

    #[test]
    fn test_hss_verify() {
        let mut private_key = HssPrivateKey::<Sha256Hasher, LEVEL>::generate(
            LmotsAlgorithm::construct_default_parameter(),
            LmsAlgorithm::construct_default_parameter(),
        )
        .expect("Should geneerate HSS private key");
        let public_key = private_key.get_public_key();

        let mut message = [42, 57, 20, 59, 33, 1, 49, 3, 99, 130, 50, 20];

        generate_signature_and_verify(&mut private_key, &public_key, &mut message);
        generate_signature_and_verify(&mut private_key, &public_key, &mut message);
    }

    fn generate_signature_and_verify<H: Hasher, const L: usize>(
        private_key: &mut HssPrivateKey<H, L>,
        public_key: &HssPublicKey<H, L>,
        message: &mut [u8],
    ) {
        let signature = HssSignature::sign(private_key, &message).expect("Should sign message");

        assert!(verify(&signature, &public_key, &message).is_ok());

        message[0] = !message[0];

        assert!(verify(&signature, &public_key, &message).is_err());
    }
}
