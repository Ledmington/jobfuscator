use classfile::classfile::ClassFile;

pub(crate) trait ClassFileTransformation {
    fn transform(&self, cf: &ClassFile) -> ClassFile;
}
