use classfile::classfile::ClassFile;

use crate::transformation::ClassFileTransformation;

pub(crate) struct ShuffleFields {}

impl ClassFileTransformation for ShuffleFields {
    fn transform(&self, cf: &ClassFile) -> ClassFile {
        todo!()
    }
}
