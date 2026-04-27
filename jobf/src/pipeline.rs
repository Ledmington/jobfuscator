use classfile::classfile::ClassFile;

use crate::transformation::ClassFileTransformation;

pub(crate) struct TransformationPipeline {
    steps: Vec<Box<dyn ClassFileTransformation>>,
}

impl TransformationPipeline {
    pub(crate) fn new() -> Self {
        TransformationPipeline { steps: vec![] }
    }

    pub(crate) fn add(&mut self, step: Box<dyn ClassFileTransformation>) {
        self.steps.push(step);
    }

    pub(crate) fn execute(&self, cf: &ClassFile) -> ClassFile {
        let mut tmp: ClassFile = cf.clone();
        for step in &self.steps {
            tmp = step.transform(&tmp);
        }
        tmp
    }
}
