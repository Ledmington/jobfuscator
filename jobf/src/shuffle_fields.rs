use classfile::classfile::ClassFile;
use rand::{SeedableRng, rngs::ChaCha8Rng, seq::SliceRandom};

use crate::transformation::ClassFileTransformation;

pub(crate) struct ShuffleFields {
    seed: u64,
}

impl ShuffleFields {
    pub fn new(seed: u64) -> Self {
        ShuffleFields { seed }
    }
}

impl ClassFileTransformation for ShuffleFields {
    fn transform(&self, cf: &ClassFile) -> ClassFile {
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);
        let mut new_fields = cf.fields.clone();
        new_fields.shuffle(&mut rng);
        ClassFile {
            minor_version: cf.minor_version,
            major_version: cf.major_version,
            constant_pool: cf.constant_pool.clone(),
            access_flags: cf.access_flags,
            this_class: cf.this_class,
            super_class: cf.super_class,
            interfaces: cf.interfaces.clone(),
            fields: new_fields,
            methods: cf.methods.clone(),
            attributes: cf.attributes.clone(),
        }
    }
}
