use std::collections::{HashMap, HashSet};

use classfile::{
    attributes::{AttributeInfo, ExceptionTableEntry},
    bytecode::BytecodeInstruction,
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
        let mut new_cp_entries: Vec<ConstantPoolInfo> = vec![ConstantPoolInfo::Null {}; cp.len()];
        assert!(new_cp_entries.len() == cp.len());
        for i in 1..=cp.len() {
            let old_cp_index: u16 = i.try_into().unwrap();
            let new_cp_index: u16 = cp_index_map.get(old_cp_index);
            let entry = &cp[old_cp_index];
            new_cp_entries[(new_cp_index - 1) as usize] = match entry {
                ConstantPoolInfo::Utf8 { .. }
                | ConstantPoolInfo::Integer { .. }
                | ConstantPoolInfo::Float { .. }
                | ConstantPoolInfo::Long { .. }
                | ConstantPoolInfo::Double { .. }
                | ConstantPoolInfo::Null {} => entry.clone(),
                ConstantPoolInfo::String { string_index } => ConstantPoolInfo::String {
                    string_index: cp_index_map.get(*string_index),
                },
                ConstantPoolInfo::Class { name_index } => ConstantPoolInfo::Class {
                    name_index: cp_index_map.get(*name_index),
                },
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
            };
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
                    code: self.modify_code(cp_index_map, code),
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

    fn modify_code(
        &self,
        cp_index_map: &CPIndexMap,
        old_code: &Vec<(u32, BytecodeInstruction)>,
    ) -> Vec<(u32, BytecodeInstruction)> {
        let mut new_code: Vec<(u32, BytecodeInstruction)> = Vec::with_capacity(old_code.len());
        for (pos, inst) in old_code.iter() {
            new_code.push((
                *pos,
                match inst {
                    BytecodeInstruction::Ldc {
                        constant_pool_index,
                    } => BytecodeInstruction::Ldc {
                        constant_pool_index: cp_index_map
                            .get(*constant_pool_index as u16)
                            .try_into()
                            .unwrap(),
                    },
                    BytecodeInstruction::LdcW {
                        constant_pool_index,
                    } => BytecodeInstruction::LdcW {
                        constant_pool_index: cp_index_map.get(*constant_pool_index),
                    },
                    BytecodeInstruction::Ldc2W {
                        constant_pool_index,
                    } => BytecodeInstruction::Ldc2W {
                        constant_pool_index: cp_index_map.get(*constant_pool_index),
                    },
                    BytecodeInstruction::ANewArray {
                        constant_pool_index,
                    } => BytecodeInstruction::ANewArray {
                        constant_pool_index: cp_index_map.get(*constant_pool_index),
                    },
                    BytecodeInstruction::New {
                        constant_pool_index,
                    } => BytecodeInstruction::New {
                        constant_pool_index: cp_index_map.get(*constant_pool_index),
                    },
                    BytecodeInstruction::GetStatic { field_ref_index } => {
                        BytecodeInstruction::GetStatic {
                            field_ref_index: cp_index_map.get(*field_ref_index),
                        }
                    }
                    BytecodeInstruction::PutStatic { field_ref_index } => {
                        BytecodeInstruction::PutStatic {
                            field_ref_index: cp_index_map.get(*field_ref_index),
                        }
                    }
                    BytecodeInstruction::GetField { field_ref_index } => {
                        BytecodeInstruction::GetField {
                            field_ref_index: cp_index_map.get(*field_ref_index),
                        }
                    }
                    BytecodeInstruction::PutField { field_ref_index } => {
                        BytecodeInstruction::PutField {
                            field_ref_index: cp_index_map.get(*field_ref_index),
                        }
                    }
                    BytecodeInstruction::InvokeSpecial { method_ref_index } => {
                        BytecodeInstruction::InvokeSpecial {
                            method_ref_index: cp_index_map.get(*method_ref_index),
                        }
                    }
                    BytecodeInstruction::InvokeStatic { method_ref_index } => {
                        BytecodeInstruction::InvokeStatic {
                            method_ref_index: cp_index_map.get(*method_ref_index),
                        }
                    }
                    BytecodeInstruction::InvokeVirtual { method_ref_index } => {
                        BytecodeInstruction::InvokeVirtual {
                            method_ref_index: cp_index_map.get(*method_ref_index),
                        }
                    }
                    BytecodeInstruction::InvokeDynamic {
                        constant_pool_index,
                    } => BytecodeInstruction::InvokeDynamic {
                        constant_pool_index: cp_index_map.get(*constant_pool_index),
                    },
                    BytecodeInstruction::InvokeInterface {
                        constant_pool_index,
                        count,
                    } => BytecodeInstruction::InvokeInterface {
                        constant_pool_index: cp_index_map.get(*constant_pool_index),
                        count: *count,
                    },
                    BytecodeInstruction::CheckCast {
                        constant_pool_index,
                    } => BytecodeInstruction::CheckCast {
                        constant_pool_index: cp_index_map.get(*constant_pool_index),
                    },
                    BytecodeInstruction::Instanceof {
                        constant_pool_index,
                    } => BytecodeInstruction::Instanceof {
                        constant_pool_index: cp_index_map.get(*constant_pool_index),
                    },

                    //
                    BytecodeInstruction::Dup {}
                    | BytecodeInstruction::Dup2 {}
                    | BytecodeInstruction::AConstNull {}
                    | BytecodeInstruction::IConst { .. }
                    | BytecodeInstruction::LConst { .. }
                    | BytecodeInstruction::FConst { .. }
                    | BytecodeInstruction::DConst { .. }
                    | BytecodeInstruction::AStore { .. }
                    | BytecodeInstruction::ILoad { .. }
                    | BytecodeInstruction::IStore { .. }
                    | BytecodeInstruction::LLoad { .. }
                    | BytecodeInstruction::LStore { .. }
                    | BytecodeInstruction::FLoad { .. }
                    | BytecodeInstruction::FStore { .. }
                    | BytecodeInstruction::DLoad { .. }
                    | BytecodeInstruction::DStore { .. }
                    | BytecodeInstruction::IaLoad {}
                    | BytecodeInstruction::LaLoad {}
                    | BytecodeInstruction::FaLoad {}
                    | BytecodeInstruction::DaLoad {}
                    | BytecodeInstruction::AaLoad {}
                    | BytecodeInstruction::BaLoad {}
                    | BytecodeInstruction::CaLoad {}
                    | BytecodeInstruction::SaLoad {}
                    | BytecodeInstruction::IaStore {}
                    | BytecodeInstruction::LaStore {}
                    | BytecodeInstruction::FaStore {}
                    | BytecodeInstruction::DaStore {}
                    | BytecodeInstruction::AaStore {}
                    | BytecodeInstruction::BaStore {}
                    | BytecodeInstruction::CaStore {}
                    | BytecodeInstruction::SaStore {}
                    | BytecodeInstruction::NewArray { .. }
                    | BytecodeInstruction::AThrow {}
                    | BytecodeInstruction::BiPush { .. }
                    | BytecodeInstruction::SiPush { .. }
                    | BytecodeInstruction::Pop {}
                    | BytecodeInstruction::Pop2 {}
                    | BytecodeInstruction::Return {}
                    | BytecodeInstruction::IReturn {}
                    | BytecodeInstruction::LReturn {}
                    | BytecodeInstruction::FReturn {}
                    | BytecodeInstruction::DReturn {}
                    | BytecodeInstruction::AReturn {}
                    | BytecodeInstruction::ArrayLength {}
                    | BytecodeInstruction::LCmp {}
                    | BytecodeInstruction::FCmpL {}
                    | BytecodeInstruction::FCmpG {}
                    | BytecodeInstruction::DCmpL {}
                    | BytecodeInstruction::DCmpG {}
                    | BytecodeInstruction::IfAcmpEq { .. }
                    | BytecodeInstruction::IfAcmpNe { .. }
                    | BytecodeInstruction::IfIcmpEq { .. }
                    | BytecodeInstruction::IfIcmpNe { .. }
                    | BytecodeInstruction::IfIcmpLt { .. }
                    | BytecodeInstruction::IfIcmpGe { .. }
                    | BytecodeInstruction::IfIcmpGt { .. }
                    | BytecodeInstruction::IfIcmpLe { .. }
                    | BytecodeInstruction::IfEq { .. }
                    | BytecodeInstruction::IfNe { .. }
                    | BytecodeInstruction::IfLt { .. }
                    | BytecodeInstruction::IfGe { .. }
                    | BytecodeInstruction::IfGt { .. }
                    | BytecodeInstruction::IfLe { .. }
                    | BytecodeInstruction::IfNull { .. }
                    | BytecodeInstruction::IfNonNull { .. }
                    | BytecodeInstruction::GoTo { .. }
                    | BytecodeInstruction::TableSwitch { .. }
                    | BytecodeInstruction::LookupSwitch { .. }
                    | BytecodeInstruction::IInc { .. }
                    | BytecodeInstruction::I2L {}
                    | BytecodeInstruction::I2F {}
                    | BytecodeInstruction::I2D {}
                    | BytecodeInstruction::L2I {}
                    | BytecodeInstruction::L2F {}
                    | BytecodeInstruction::L2D {}
                    | BytecodeInstruction::F2I {}
                    | BytecodeInstruction::F2L {}
                    | BytecodeInstruction::F2D {}
                    | BytecodeInstruction::D2I {}
                    | BytecodeInstruction::D2L {}
                    | BytecodeInstruction::D2F {}
                    | BytecodeInstruction::I2B {}
                    | BytecodeInstruction::I2C {}
                    | BytecodeInstruction::I2S {}
                    | BytecodeInstruction::IAdd {}
                    | BytecodeInstruction::ISub {}
                    | BytecodeInstruction::IMul {}
                    | BytecodeInstruction::IDiv {}
                    | BytecodeInstruction::IRem {}
                    | BytecodeInstruction::IAnd {}
                    | BytecodeInstruction::IShl {}
                    | BytecodeInstruction::IShr {}
                    | BytecodeInstruction::IUshr {}
                    | BytecodeInstruction::IOr {}
                    | BytecodeInstruction::IXor {}
                    | BytecodeInstruction::INeg {}
                    | BytecodeInstruction::LAdd {}
                    | BytecodeInstruction::LSub {}
                    | BytecodeInstruction::LMul {}
                    | BytecodeInstruction::LDiv {}
                    | BytecodeInstruction::LRem {}
                    | BytecodeInstruction::LAnd {}
                    | BytecodeInstruction::LOr {}
                    | BytecodeInstruction::LXor {}
                    | BytecodeInstruction::LShl {}
                    | BytecodeInstruction::LShr {}
                    | BytecodeInstruction::LUshr {}
                    | BytecodeInstruction::LNeg {}
                    | BytecodeInstruction::FAdd {}
                    | BytecodeInstruction::FMul {}
                    | BytecodeInstruction::FNeg {}
                    | BytecodeInstruction::FDiv {}
                    | BytecodeInstruction::FRem {}
                    | BytecodeInstruction::FSub {}
                    | BytecodeInstruction::DAdd {}
                    | BytecodeInstruction::DMul {}
                    | BytecodeInstruction::DNeg {}
                    | BytecodeInstruction::DDiv {}
                    | BytecodeInstruction::DRem {}
                    | BytecodeInstruction::DSub {}
                    | BytecodeInstruction::ALoad { .. } => inst.clone(),
                },
            ));
        }
        new_code
    }
}

/// A structure to map old CP indices to new CP indices.
struct CPIndexMap {
    /// Internal mapping of constant pool indices: uses range [[ `1` ; `cp.len()` ]].
    map: HashMap<u16, u16>,
}

impl CPIndexMap {
    /// The input index is assumed to be in the range [[ `1` ; `cp.len()` ]].
    fn get(&self, old_cp_index: u16) -> u16 {
        assert!(old_cp_index >= 1);
        *self.map.get(&old_cp_index).unwrap()
    }
}

impl ClassFileTransformation for ShuffleConstantPool {
    fn transform(&self, cf: &ClassFile) -> ClassFile {
        let cp_index_map: CPIndexMap = self.shuffle_indices(&cf.constant_pool);

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
