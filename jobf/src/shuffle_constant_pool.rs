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

    fn shuffle_indices(&self, cp: &ConstantPool) -> HashMap<u16, u16> {
        // We cannot just shuffle all the indices, we need to shuffle just the indices of all entries that are not NULL,
        // Then re-insert them after LONG and DOUBLE entries.

        let mut new_cp_indices: Vec<u16> = (0..cp.entries.len())
            .into_iter()
            .filter(|idx| !matches!(cp[(*idx).try_into().unwrap()], ConstantPoolInfo::Null {}))
            .map(|idx| idx.try_into().unwrap())
            .collect();

        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);
        new_cp_indices.shuffle(&mut rng);

        let mut cp_index_map: HashMap<u16, u16> = HashMap::new();
        for i in 0..cp.entries.len() {
            cp_index_map.insert(i.try_into().unwrap(), new_cp_indices[i].try_into().unwrap());
        }
        assert!(cp_index_map.len() == new_cp_indices.len());

        {
            let mut index: u16 = 0;
            while index < new_cp_indices.len().try_into().unwrap() {
                if matches!(cp[index], ConstantPoolInfo::Long { .. })
                    || matches!(cp[index], ConstantPoolInfo::Double { .. })
                {
                    cp_index_map.insert(cp_index_map.get(&index).unwrap() + 1, index + 1);

                    index += 2;
                } else {
                    index += 1;
                }
            }
        }

        assert!(cp_index_map.len() == cp.len());

        return cp_index_map;
    }

    fn modify_constant_pool(
        &self,
        cp_index_map: &HashMap<u16, u16>,
        cp: &ConstantPool,
    ) -> ConstantPool {
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
                    string_index: *cp_index_map.get(string_index).unwrap(),
                },
                ConstantPoolInfo::Class { name_index } => ConstantPoolInfo::Class {
                    name_index: *cp_index_map.get(name_index).unwrap(),
                },
                ConstantPoolInfo::FieldRef {
                    class_index,
                    name_and_type_index,
                } => ConstantPoolInfo::FieldRef {
                    class_index: *cp_index_map.get(class_index).unwrap(),
                    name_and_type_index: *cp_index_map.get(name_and_type_index).unwrap(),
                },
                ConstantPoolInfo::MethodRef {
                    class_index,
                    name_and_type_index,
                } => ConstantPoolInfo::MethodRef {
                    class_index: *cp_index_map.get(class_index).unwrap(),
                    name_and_type_index: *cp_index_map.get(name_and_type_index).unwrap(),
                },
                ConstantPoolInfo::InterfaceMethodRef {
                    class_index,
                    name_and_type_index,
                } => ConstantPoolInfo::InterfaceMethodRef {
                    class_index: *cp_index_map.get(class_index).unwrap(),
                    name_and_type_index: *cp_index_map.get(name_and_type_index).unwrap(),
                },
                ConstantPoolInfo::NameAndType {
                    name_index,
                    descriptor_index,
                } => ConstantPoolInfo::NameAndType {
                    name_index: *cp_index_map.get(name_index).unwrap(),
                    descriptor_index: *cp_index_map.get(descriptor_index).unwrap(),
                },
                ConstantPoolInfo::MethodType { descriptor_index } => ConstantPoolInfo::MethodType {
                    descriptor_index: *cp_index_map.get(descriptor_index).unwrap(),
                },
                ConstantPoolInfo::MethodHandle {
                    reference_kind,
                    reference_index,
                } => ConstantPoolInfo::MethodHandle {
                    reference_kind: *reference_kind,
                    reference_index: *cp_index_map.get(reference_index).unwrap(),
                },
                ConstantPoolInfo::InvokeDynamic {
                    bootstrap_method_attr_index,
                    name_and_type_index,
                } => ConstantPoolInfo::InvokeDynamic {
                    bootstrap_method_attr_index: *cp_index_map
                        .get(bootstrap_method_attr_index)
                        .unwrap(),
                    name_and_type_index: *cp_index_map.get(name_and_type_index).unwrap(),
                },
            });
        }
        return ConstantPool {
            entries: new_cp_entries,
        };
    }

    fn modify_fields(
        &self,
        cp_index_map: &HashMap<u16, u16>,
        cp: &ConstantPool,
        fields: &Vec<FieldInfo>,
    ) -> Vec<FieldInfo> {
        let mut new_fields = Vec::with_capacity(fields.len());
        for field in fields {
            new_fields.push(FieldInfo {
                access_flags: field.access_flags,
                name_index: *cp_index_map.get(&field.name_index).unwrap(),
                descriptor_index: *cp_index_map.get(&field.descriptor_index).unwrap(),
                attributes: self.modify_attributes(cp_index_map, cp, &field.attributes),
            });
        }
        return new_fields;
    }

    fn modify_methods(
        &self,
        cp_index_map: &HashMap<u16, u16>,
        cp: &ConstantPool,
        methods: &Vec<MethodInfo>,
    ) -> Vec<MethodInfo> {
        let mut new_methods = Vec::with_capacity(methods.len());
        for method in methods {
            new_methods.push(MethodInfo {
                access_flags: method.access_flags,
                name_index: *cp_index_map.get(&method.name_index).unwrap(),
                descriptor_index: *cp_index_map.get(&method.descriptor_index).unwrap(),
                attributes: self.modify_attributes(cp_index_map, cp, &method.attributes),
            });
        }
        return new_methods;
    }

    fn modify_attributes(
        &self,
        cp_index_map: &HashMap<u16, u16>,
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
                    name_index: *cp_index_map.get(name_index).unwrap(),
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
                    name_index: *cp_index_map.get(name_index).unwrap(),
                    line_number_table: line_number_table.clone(),
                },
                AttributeInfo::LocalVariableTable {
                    name_index,
                    local_variable_table,
                } => AttributeInfo::LocalVariableTable {
                    name_index: *cp_index_map.get(name_index).unwrap(),
                    local_variable_table: local_variable_table.clone(),
                },
                AttributeInfo::LocalVariableTypeTable {
                    name_index,
                    local_variable_type_table,
                } => todo!(),
                AttributeInfo::StackMapTable {
                    name_index,
                    stack_map_table,
                } => AttributeInfo::StackMapTable {
                    name_index: *cp_index_map.get(name_index).unwrap(),
                    stack_map_table: stack_map_table.clone(),
                },
                AttributeInfo::SourceFile {
                    name_index,
                    source_file_index,
                } => AttributeInfo::SourceFile {
                    name_index: *cp_index_map.get(name_index).unwrap(),
                    source_file_index: *cp_index_map.get(source_file_index).unwrap(),
                },
                AttributeInfo::BootstrapMethods {
                    name_index,
                    methods,
                } => AttributeInfo::BootstrapMethods {
                    name_index: *cp_index_map.get(name_index).unwrap(),
                    methods: methods.clone(),
                },
                AttributeInfo::InnerClasses {
                    name_index,
                    classes,
                } => AttributeInfo::InnerClasses {
                    name_index: *cp_index_map.get(name_index).unwrap(),
                    classes: classes.clone(),
                },
                AttributeInfo::MethodParameters {
                    name_index,
                    parameters,
                } => todo!(),
                AttributeInfo::Record {
                    name_index,
                    components,
                } => todo!(),
                AttributeInfo::Signature {
                    name_index,
                    signature_index,
                } => todo!(),
                AttributeInfo::NestMembers {
                    name_index,
                    classes,
                } => todo!(),
                AttributeInfo::RuntimeVisibleAnnotations {
                    name_index,
                    annotations,
                } => todo!(),
                AttributeInfo::ConstantValue {
                    name_index,
                    constant_value_index,
                } => AttributeInfo::ConstantValue {
                    name_index: *cp_index_map.get(name_index).unwrap(),
                    constant_value_index: *cp_index_map.get(constant_value_index).unwrap(),
                },
                AttributeInfo::Exceptions {
                    name_index,
                    exception_indices,
                } => todo!(),
                AttributeInfo::EnclosingMethod {
                    name_index,
                    class_index,
                    method_index,
                } => AttributeInfo::EnclosingMethod {
                    name_index: *cp_index_map.get(name_index).unwrap(),
                    class_index: *cp_index_map.get(class_index).unwrap(),
                    method_index: *cp_index_map.get(method_index).unwrap(),
                },
                AttributeInfo::NestHost {
                    name_index,
                    host_class_index,
                } => AttributeInfo::NestHost {
                    name_index: *cp_index_map.get(name_index).unwrap(),
                    host_class_index: *cp_index_map.get(host_class_index).unwrap(),
                },
            });
        }
        return new_attributes;
    }
}

impl ClassFileTransformation for ShuffleConstantPool {
    fn transform(&self, cf: &ClassFile) -> ClassFile {
        let cp_index_map: HashMap<u16, u16> = self.shuffle_indices(&cf.constant_pool);

        ClassFile {
            minor_version: cf.minor_version,
            major_version: cf.major_version,
            constant_pool: self.modify_constant_pool(&cp_index_map, &cf.constant_pool),
            access_flags: cf.access_flags,
            this_class: *cp_index_map.get(&cf.this_class).unwrap(),
            super_class: *cp_index_map.get(&cf.super_class).unwrap(),
            interfaces: cf
                .interfaces
                .iter()
                .map(|interface_index| *cp_index_map.get(interface_index).unwrap())
                .collect(),
            fields: self.modify_fields(&cp_index_map, &cf.constant_pool, &cf.fields),
            methods: self.modify_methods(&cp_index_map, &cf.constant_pool, &cf.methods),
            attributes: self.modify_attributes(&cp_index_map, &cf.constant_pool, &cf.attributes),
        }
    }
}
