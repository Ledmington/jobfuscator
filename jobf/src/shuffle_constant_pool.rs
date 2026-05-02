use std::collections::HashMap;

use classfile::{
    attributes::AttributeInfo,
    classfile::ClassFile,
    constant_pool::{ConstantPool, ConstantPoolInfo},
    fields::FieldInfo,
    methods::MethodInfo,
};
use rand::{SeedableRng, rngs::ChaCha8Rng, seq::SliceRandom};

use crate::transformation::ClassFileTransformation;

pub(crate) struct ShuffleConstantPool {
    seed: u64,
}

impl ShuffleConstantPool {
    pub fn new(seed: u64) -> Self {
        ShuffleConstantPool { seed }
    }

    fn shuffle_indices(&self, cp: &ConstantPool) -> CPIndexMap {
        // We cannot just shuffle all the indices, we need to shuffle just the indices of all entries that are not NULL,
        // Then re-insert them after LONG and DOUBLE entries.

        let non_null_indices: Vec<u16> = (0..cp.entries.len())
            .filter(|idx| !matches!(cp[(*idx).try_into().unwrap()], ConstantPoolInfo::Null {}))
            .map(|idx| idx as u16)
            .collect();

        let mut shuffled = non_null_indices.clone();
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);
        shuffled.shuffle(&mut rng);

        // Map old index -> new index
        let mut cp_index_map: HashMap<u16, u16> = HashMap::new();
        for (new_pos, &old_idx) in shuffled.iter().enumerate() {
            cp_index_map.insert(old_idx, new_pos.try_into().unwrap());
        }

        // Fix up the implicit Null slots after Long/Double
        // Wherever old_idx is a Long/Double, old_idx+1 must follow new_idx+1
        for old_idx in 0..cp.entries.len().try_into().unwrap() {
            if matches!(
                cp[old_idx],
                ConstantPoolInfo::Long { .. } | ConstantPoolInfo::Double { .. }
            ) {
                let new_idx = *cp_index_map.get(&old_idx).unwrap();
                cp_index_map.insert(old_idx + 1, new_idx + 1);
            }
        }

        CPIndexMap { map: cp_index_map }
    }

    fn modify_constant_pool(&self, cp_index_map: &CPIndexMap, cp: &ConstantPool) -> ConstantPool {
        let mut new_cp_entries: Vec<ConstantPoolInfo> = Vec::with_capacity(cp.len());
        for entry in cp.entries.iter() {
            new_cp_entries.push(match entry {
                ConstantPoolInfo::Utf8 { .. }
                | ConstantPoolInfo::Integer { .. }
                | ConstantPoolInfo::Float { .. }
                | ConstantPoolInfo::Long { .. }
                | ConstantPoolInfo::Double { .. }
                | ConstantPoolInfo::Null {} => entry.clone(),
                ConstantPoolInfo::String { string_index } => ConstantPoolInfo::String {
                    string_index: cp_index_map.get(*string_index),
                },
                ConstantPoolInfo::Class { name_index } => {
                    println!(
                        "Class entry ('{}'): from {} to {}",
                        cp.get_class_name(*name_index - 1),
                        name_index,
                        cp_index_map.get(*name_index)
                    );
                    ConstantPoolInfo::Class {
                        name_index: cp_index_map.get(*name_index),
                    }
                }
                ConstantPoolInfo::FieldRef {
                    class_index,
                    name_and_type_index,
                } => ConstantPoolInfo::FieldRef {
                    class_index: cp_index_map.get(*class_index),
                    name_and_type_index: cp_index_map.get(*name_and_type_index),
                },
                ConstantPoolInfo::MethodRef {
                    class_index,
                    name_and_type_index,
                } => ConstantPoolInfo::MethodRef {
                    class_index: cp_index_map.get(*class_index),
                    name_and_type_index: cp_index_map.get(*name_and_type_index),
                },
                ConstantPoolInfo::InterfaceMethodRef {
                    class_index,
                    name_and_type_index,
                } => ConstantPoolInfo::InterfaceMethodRef {
                    class_index: cp_index_map.get(*class_index),
                    name_and_type_index: cp_index_map.get(*name_and_type_index),
                },
                ConstantPoolInfo::NameAndType {
                    name_index,
                    descriptor_index,
                } => ConstantPoolInfo::NameAndType {
                    name_index: cp_index_map.get(*name_index),
                    descriptor_index: cp_index_map.get(*descriptor_index),
                },
                ConstantPoolInfo::MethodType { descriptor_index } => ConstantPoolInfo::MethodType {
                    descriptor_index: cp_index_map.get(*descriptor_index),
                },
                ConstantPoolInfo::MethodHandle {
                    reference_kind,
                    reference_index,
                } => ConstantPoolInfo::MethodHandle {
                    reference_kind: *reference_kind,
                    reference_index: cp_index_map.get(*reference_index),
                },
                ConstantPoolInfo::InvokeDynamic {
                    bootstrap_method_attr_index,
                    name_and_type_index,
                } => ConstantPoolInfo::InvokeDynamic {
                    bootstrap_method_attr_index: *bootstrap_method_attr_index,
                    name_and_type_index: cp_index_map.get(*name_and_type_index),
                },
            });
        }
        return ConstantPool {
            entries: new_cp_entries,
        };
    }

    fn modify_fields(
        &self,
        cp_index_map: &CPIndexMap,
        cp: &ConstantPool,
        fields: &Vec<FieldInfo>,
    ) -> Vec<FieldInfo> {
        let mut new_fields = Vec::with_capacity(fields.len());
        for field in fields {
            new_fields.push(FieldInfo {
                access_flags: field.access_flags,
                name_index: cp_index_map.get(field.name_index),
                descriptor_index: cp_index_map.get(field.descriptor_index),
                attributes: self.modify_attributes(cp_index_map, cp, &field.attributes),
            });
        }
        return new_fields;
    }

    fn modify_methods(
        &self,
        cp_index_map: &CPIndexMap,
        cp: &ConstantPool,
        methods: &Vec<MethodInfo>,
    ) -> Vec<MethodInfo> {
        let mut new_methods = Vec::with_capacity(methods.len());
        for method in methods {
            new_methods.push(MethodInfo {
                access_flags: method.access_flags,
                name_index: cp_index_map.get(method.name_index),
                descriptor_index: cp_index_map.get(method.descriptor_index),
                attributes: self.modify_attributes(cp_index_map, cp, &method.attributes),
            });
        }
        return new_methods;
    }

    fn modify_attributes(
        &self,
        cp_index_map: &CPIndexMap,
        cp: &ConstantPool,
        attributes: &Vec<AttributeInfo>,
    ) -> Vec<AttributeInfo> {
        let mut new_attributes: Vec<AttributeInfo> = Vec::with_capacity(attributes.len());
        for attribute in attributes {
            new_attributes.push(match attribute {
                AttributeInfo::Code {
                    name_index,
                    max_stack,
                    max_locals,
                    code,
                    exception_table,
                    attributes,
                } => AttributeInfo::Code {
                    name_index: cp_index_map.get(*name_index),
                    max_stack: *max_stack,
                    max_locals: *max_locals,
                    code: code.clone(),
                    exception_table: exception_table.clone(),
                    attributes: self.modify_attributes(cp_index_map, cp, attributes),
                },
                AttributeInfo::LineNumberTable {
                    name_index,
                    line_number_table,
                } => AttributeInfo::LineNumberTable {
                    name_index: cp_index_map.get(*name_index),
                    line_number_table: line_number_table.clone(),
                },
                AttributeInfo::LocalVariableTable {
                    name_index,
                    local_variable_table,
                } => AttributeInfo::LocalVariableTable {
                    name_index: cp_index_map.get(*name_index),
                    local_variable_table: local_variable_table.clone(),
                },
                AttributeInfo::LocalVariableTypeTable {
                    name_index,
                    local_variable_type_table,
                } => AttributeInfo::LocalVariableTypeTable {
                    name_index: cp_index_map.get(*name_index),
                    local_variable_type_table: local_variable_type_table.clone(),
                },
                AttributeInfo::StackMapTable {
                    name_index,
                    stack_map_table,
                } => AttributeInfo::StackMapTable {
                    name_index: cp_index_map.get(*name_index),
                    stack_map_table: stack_map_table.clone(),
                },
                AttributeInfo::SourceFile {
                    name_index,
                    source_file_index,
                } => AttributeInfo::SourceFile {
                    name_index: cp_index_map.get(*name_index),
                    source_file_index: cp_index_map.get(*source_file_index),
                },
                AttributeInfo::BootstrapMethods {
                    name_index,
                    methods,
                } => AttributeInfo::BootstrapMethods {
                    name_index: cp_index_map.get(*name_index),
                    methods: methods.clone(),
                },
                AttributeInfo::InnerClasses {
                    name_index,
                    classes,
                } => AttributeInfo::InnerClasses {
                    name_index: cp_index_map.get(*name_index),
                    classes: classes.clone(),
                },
                AttributeInfo::MethodParameters {
                    name_index,
                    parameters,
                } => AttributeInfo::MethodParameters {
                    name_index: cp_index_map.get(*name_index),
                    parameters: parameters.clone(),
                },
                AttributeInfo::Record {
                    name_index,
                    components,
                } => AttributeInfo::Record {
                    name_index: cp_index_map.get(*name_index),
                    components: components.clone(),
                },
                AttributeInfo::Signature {
                    name_index,
                    signature_index,
                } => AttributeInfo::Signature {
                    name_index: cp_index_map.get(*name_index),
                    signature_index: cp_index_map.get(*signature_index),
                },
                AttributeInfo::NestMembers {
                    name_index,
                    classes,
                } => AttributeInfo::NestMembers {
                    name_index: cp_index_map.get(*name_index),
                    classes: classes
                        .iter()
                        .map(|class_idx| cp_index_map.get(*class_idx))
                        .collect(),
                },
                AttributeInfo::RuntimeVisibleAnnotations {
                    name_index,
                    annotations,
                } => AttributeInfo::RuntimeVisibleAnnotations {
                    name_index: cp_index_map.get(*name_index),
                    annotations: annotations.clone(),
                },
                AttributeInfo::ConstantValue {
                    name_index,
                    constant_value_index,
                } => AttributeInfo::ConstantValue {
                    name_index: cp_index_map.get(*name_index),
                    constant_value_index: cp_index_map.get(*constant_value_index),
                },
                AttributeInfo::Exceptions {
                    name_index,
                    exception_indices,
                } => AttributeInfo::Exceptions {
                    name_index: cp_index_map.get(*name_index),
                    exception_indices: exception_indices
                        .iter()
                        .map(|exception_index| cp_index_map.get(*exception_index))
                        .collect(),
                },
                AttributeInfo::EnclosingMethod {
                    name_index,
                    class_index,
                    method_index,
                } => AttributeInfo::EnclosingMethod {
                    name_index: cp_index_map.get(*name_index),
                    class_index: cp_index_map.get(*class_index),
                    method_index: cp_index_map.get(*method_index),
                },
                AttributeInfo::NestHost {
                    name_index,
                    host_class_index,
                } => AttributeInfo::NestHost {
                    name_index: cp_index_map.get(*name_index),
                    host_class_index: cp_index_map.get(*host_class_index),
                },
            });
        }
        return new_attributes;
    }
}

struct CPIndexMap {
    /// Internal mapping of constant pool indices: uses range [[ `0` ; `cp.len()-1` ]].
    map: HashMap<u16, u16>,
}

impl CPIndexMap {
    /// The input index is assumed to be in the range [[ `1` ; `cp.len()` ]].
    fn get(&self, cp_index: u16) -> u16 {
        assert!(cp_index >= 1);
        self.map.get(&(cp_index - 1)).unwrap() + 1
    }
}

impl ClassFileTransformation for ShuffleConstantPool {
    fn transform(&self, cf: &ClassFile) -> ClassFile {
        let cp_index_map: CPIndexMap = self.shuffle_indices(&cf.constant_pool);

        {
            let mut keys: Vec<&u16> = cp_index_map.map.keys().collect();
            keys.sort();
            for k in keys.iter() {
                println!(" {}: {}", *k + 1, cp_index_map.get(**k + 1));
            }
        }

        ClassFile {
            minor_version: cf.minor_version,
            major_version: cf.major_version,
            constant_pool: self.modify_constant_pool(&cp_index_map, &cf.constant_pool),
            access_flags: cf.access_flags,
            this_class: cp_index_map.get(cf.this_class),
            super_class: if cf.super_class == 0 {
                0
            } else {
                cp_index_map.get(cf.super_class)
            },
            interfaces: cf
                .interfaces
                .iter()
                .map(|interface_index| cp_index_map.get(*interface_index))
                .collect(),
            fields: self.modify_fields(&cp_index_map, &cf.constant_pool, &cf.fields),
            methods: self.modify_methods(&cp_index_map, &cf.constant_pool, &cf.methods),
            attributes: self.modify_attributes(&cp_index_map, &cf.constant_pool, &cf.attributes),
        }
    }
}
