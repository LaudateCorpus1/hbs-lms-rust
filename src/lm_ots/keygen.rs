use super::definitions::*;
use super::parameter::LmotsParameter;
use crate::hasher::Hasher;
use crate::util::dynamic_array::DynamicArray;
use crate::{
    constants::{D_PBLC, MAX_N, MAX_P},
    util::ustr::*,
};

pub fn generate_private_key<P: LmotsParameter>(
    i: IType,
    q: QType,
    seed: Seed,
) -> LmotsPrivateKey<P> {
    let mut key = DynamicArray::new();

    let mut lmots_parameter = <P>::new();
    let mut hasher = &lmots_parameter;

    for index in 0..lmots_parameter.get_p() {
        hasher.update(&i);
        hasher.update(&q);
        hasher.update(&u16str(index as u16));
        hasher.update(&[0xff]);
        hasher.update(&seed);

        key[index as usize] = hasher.finalize_reset();
    }

    LmotsPrivateKey::new(i, q, lmots_parameter, key)
}

pub fn generate_public_key<P: LmotsParameter>(
    private_key: &LmotsPrivateKey<P>,
) -> LmotsPublicKey<P> {
    let parameter = &private_key.parameter;

    let max_word_index: usize = (1 << parameter.w) - 1;
    let key = &private_key.key;

    let mut hasher = parameter.get_hasher();

    let mut y: DynamicArray<DynamicArray<u8, MAX_N>, MAX_P> = DynamicArray::new();

    for i in 0..parameter.p as usize {
        let mut tmp = key[i];

        for j in 0..max_word_index {
            hasher.update(&private_key.I);
            hasher.update(&private_key.q);
            hasher.update(&u16str(i as u16));
            hasher.update(&u8str(j as u8));
            hasher.update(tmp.get_slice());

            for (index, value) in hasher.finalize_reset().into_iter().enumerate() {
                tmp[index] = value;
            }
        }

        y[i] = tmp;
    }

    hasher.update(&private_key.I);
    hasher.update(&private_key.q);
    hasher.update(&D_PBLC);

    for item in y.into_iter() {
        hasher.update(item.get_slice());
    }

    let mut public_key = DynamicArray::new();
    for (index, value) in hasher.finalize().into_iter().enumerate() {
        public_key[index] = value;
    }

    LmotsPublicKey::new(private_key.I, private_key.q, public_key)
}
