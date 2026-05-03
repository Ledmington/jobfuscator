use std::collections::{HashMap, HashSet};

use classfile::{
    attributes::{AttributeInfo, ExceptionTableEntry},
    classfile::ClassFile,
    constant_pool::{ConstantPool, ConstantPoolInfo},
    fields::FieldInfo,
    methods::MethodInfo,
};
use rand::{RngExt, SeedableRng, rngs::ChaCha8Rng};

use crate::transformation::ClassFileTransformation;

pub(crate) struct ShuffleConstantPool {
    seed: u64,
}

impl ShuffleConstantPool {
    pub fn new(seed: u64) -> Self {
        ShuffleConstantPool { seed }
    }

    fn shuffle_indices(&self, cp: &ConstantPool) -> CPIndexMap {
        // We cannot just shuffle all the indices, we need to shuffle just the indices of all entries that are not NULL.

        let mut indices: Vec<u16> = (1..=(cp.len().try_into().unwrap())).collect();
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);

        // Actual shuffle (Fisher-Yates-like)
        let mut i = indices.len() - 1;
        while i > 0 {
            // Skip null slots (they'll be dragged along by their Long/Double)
            if matches!(cp[indices[i]], ConstantPoolInfo::Null {}) {
                i -= 1;
                continue;
            }

            let i_is_special = matches!(
                cp[indices[i]],
                ConstantPoolInfo::Long { .. } | ConstantPoolInfo::Double { .. }
            );

            loop {
                let j = rng.random_range(0..i);

                let j_is_null = matches!(cp[indices[j]], ConstantPoolInfo::Null {});
                let j_is_special = matches!(
                    cp[indices[j]],
                    ConstantPoolInfo::Long { .. } | ConstantPoolInfo::Double { .. }
                );

                if j_is_null
                    || (i_is_special && j + 1 == i)
                    || (j_is_special && i == j + 1)
                    || (j_is_special && i == indices.len() - 1)
                {
                    continue;
                }

                indices.swap(i, j);
                if i_is_special || j_is_special {
                    indices.swap(i + 1, j + 1);
                }
                break;
            }

            // Skip over the null slot if i was a Long/Double pair
            if i_is_special {
                i -= 1;
            }
            i -= 1;
        }

        // Map old index -> new index
        let mut cp_index_map: HashMap<u16, u16> = HashMap::new();
        for (new_pos, &old_idx) in indices.iter().enumerate() {
            // `old_idx` is 1-based, `new_pos` is 0-based
            cp_index_map.insert(old_idx, (new_pos + 1).try_into().unwrap());
        }

        {
            // TODO: only for debug, remove
            println!(" ### CP_INDEX_MAP ### ");
            for (k, v) in cp_index_map.iter() {
                println!("  {k} : {v}");
            }
            println!(" ### CP_INDEX_MAP ### ");
        }

        // Make sure that the CP index map does not contain excess elements
        assert!(cp_index_map.len() == cp.len());

        // Make sure that the CP index map contains a mapping for each starting index
        assert!((1..=cp.len()).all(|k| cp_index_map.contains_key(&k.try_into().unwrap())));

        // Make sure that the CP index map can map to each new index
        {
            let values: HashSet<&u16> = cp_index_map.values().collect();
            assert!(values.len() == cp.len());
            assert!((1..=cp.len()).all(|v| values.contains(&(v as u16))));
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
                    println!(" ### ");
                    println!(" OLD : '{}'", cp.get_class_name(*name_index));
                    println!(" {} : {}", name_index - 1, cp[name_index - 1].tag());
                    println!(" {} : {}", name_index, cp[*name_index].tag());
                    println!(" {} : {}", name_index + 1, cp[name_index + 1].tag());
                    println!(" ### ");
                    // println!(
                    //     "Class entry ('{}'): from {} to {}",
                    //     cp.get_class_name(*name_index),
                    //     name_index,
                    //     cp_index_map.get(*name_index)
                    // );
                    ConstantPoolInfo::Class {
                        name_index: cp_index_map.get(*name_index - 1),
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
        ConstantPool {
            entries: new_cp_entries,
        }
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
        new_fields
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
        new_methods
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
                    exception_table: exception_table
                        .iter()
                        .map(|exc_entry| ExceptionTableEntry {
                            start_pc: exc_entry.start_pc,
                            end_pc: exc_entry.end_pc,
                            handler_pc: exc_entry.handler_pc,
                            catch_type: if exc_entry.catch_type == 0 {
                                0
                            } else {
                                cp_index_map.get(exc_entry.catch_type)
                            },
                        })
                        .collect(),
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
                AttributeInfo::Deprecated { name_index } => AttributeInfo::Deprecated {
                    name_index: cp_index_map.get(*name_index),
                },
            });
        }
        new_attributes
    }
}

/// A structure to map old CP indices to new CP indices.
struct CPIndexMap {
    /// Internal mapping of constant pool indices: uses range [[ `0` ; `cp.len()-1` ]].
    map: HashMap<u16, u16>,
}

impl CPIndexMap {
    /// The input index is assumed to be in the range [[ `1` ; `cp.len()` ]].
    fn get(&self, old_cp_index: u16) -> u16 {
        assert!(old_cp_index >= 1);
        self.map.get(&(old_cp_index - 1)).unwrap() + 1
    }
}

impl ClassFileTransformation for ShuffleConstantPool {
    fn transform(&self, cf: &ClassFile) -> ClassFile {
        let cp_index_map: CPIndexMap = self.shuffle_indices(&cf.constant_pool);

        {
            println!(" ### CP_INDEX_MAP ### ");
            let mut pairs: Vec<(&u16, &u16)> = cp_index_map.map.iter().collect();
            pairs.sort_by_key(|&(&k, _)| k);
            for (k, v) in pairs {
                println!("  {k} : {v}");
            }
            println!(" ### CP_INDEX_MAP ### ");
        }

        let new_constant_pool: ConstantPool =
            self.modify_constant_pool(&cp_index_map, &cf.constant_pool);

        let new_this_class = cp_index_map.get(cf.this_class);
        let new_super_class = if cf.super_class == 0 {
            0
        } else {
            cp_index_map.get(cf.super_class)
        };
        let new_interfaces = cf
            .interfaces
            .iter()
            .map(|interface_index| cp_index_map.get(*interface_index))
            .collect();
        let new_fields = self.modify_fields(&cp_index_map, &cf.constant_pool, &cf.fields);
        let new_methods = self.modify_methods(&cp_index_map, &cf.constant_pool, &cf.methods);
        let new_attributes =
            self.modify_attributes(&cp_index_map, &cf.constant_pool, &cf.attributes);

        ClassFile {
            minor_version: cf.minor_version,
            major_version: cf.major_version,
            constant_pool: new_constant_pool,
            access_flags: cf.access_flags,
            this_class: new_this_class,
            super_class: new_super_class,
            interfaces: new_interfaces,
            fields: new_fields,
            methods: new_methods,
            attributes: new_attributes,
        }
    }
}
