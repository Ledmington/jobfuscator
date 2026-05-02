use classfile::{classfile::ClassFile, constant_pool::ConstantPool};
use rand::{SeedableRng, rngs::ChaCha8Rng, seq::SliceRandom};

use crate::transformation::ClassFileTransformation;

pub(crate) struct ShuffleConstantPool {
    seed: u64,
}

impl ShuffleConstantPool {
    pub fn new(seed: u64) -> Self {
        ShuffleConstantPool { seed }
    }
}

impl ClassFileTransformation for ShuffleConstantPool {
    fn transform(&self, cf: &ClassFile) -> ClassFile {
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);
        let mut new_cp_entries = cf.constant_pool.clone().entries;
        new_cp_entries.shuffle(&mut rng);
        ClassFile {
            minor_version: cf.minor_version,
            major_version: cf.major_version,
            constant_pool: ConstantPool {
                entries: new_cp_entries,
            },
            access_flags: cf.access_flags,
            this_class: cf.this_class,
            super_class: cf.super_class,
            interfaces: cf.interfaces.clone(),
            fields: cf.fields.clone(),
            methods: cf.methods.clone(),
            attributes: cf.attributes.clone(),
        }
    }
}
