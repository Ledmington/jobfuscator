use std::collections::{BTreeMap, HashMap, HashSet};

use classfile::{
    attributes::{AttributeInfo, AttributeKind, ExceptionTableEntry, find_attribute},
    bytecode::BytecodeInstruction,
    classfile::ClassFile,
    constant_pool::{ConstantPool, ConstantPoolInfo},
    fields::FieldInfo,
    methods::MethodInfo,
};
use rand::{RngExt, SeedableRng, rngs::ChaCha8Rng};

use crate::transformation::ClassFileTransformation;

fn collect_special_indices(cf: &ClassFile) -> HashSet<u16> {
    // Collect all "special" indices to fix later (instructions like ldc require a u8 constant_pool_index,
    // meaning that their argument must fit into a byte, therefore the mapped index must be <256)
    let mut special_indices: HashSet<u16> = HashSet::new();
    for method in cf.methods.iter() {
        let attr = find_attribute(&method.attributes, AttributeKind::Code);
        if let Some(AttributeInfo::Code { code, .. }) = attr {
            for (_, inst) in code {
                if let BytecodeInstruction::Ldc {
                    constant_pool_index,
                } = inst
                {
                    special_indices.insert(*constant_pool_index as u16);
                }
            }
        }
    }
    special_indices
}

fn shuffle_indices(
    seed: u64,
    indices: &HashSet<u16>,
    special_indices: &HashSet<u16>,
) -> CPIndexMap {
    assert!(
        !indices.contains(&0),
        "Constant Pool indices to be shuffled are expected to be 1-based."
    );
    assert!(
        special_indices
            .iter()
            .all(|special_index| indices.contains(special_index)),
        "All special indices must be contained within indices."
    );

    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    // linearize indices
    let mut indices_vec: Vec<u16> = indices.iter().copied().collect();
    indices_vec.sort();

    // one pass of Fisher-Yates
    for i in 0..(indices_vec.len() - 2) {
        let j = rng.random_range(i..=(indices_vec.len() - 1));
        indices_vec.swap(i, j);
    }

    if !special_indices.is_empty() {
        todo!("Don't know what to do with special indices");
    }

    // build the index map
    let mut new_indices = HashMap::new();
    for (new_idx, old_idx) in indices_vec.iter().enumerate() {
        new_indices.insert(*old_idx, (new_idx + 1).try_into().unwrap());
    }

    CPIndexMap { map: new_indices }
}

pub(crate) struct ShuffleConstantPool {
    seed: u64,
}

impl ShuffleConstantPool {
    pub fn new(seed: u64) -> Self {
        ShuffleConstantPool { seed }
    }

    fn modify_constant_pool(&self, cp_index_map: &CPIndexMap, cp: &ConstantPool) -> ConstantPool {
        let mut new_cp_entries: BTreeMap<u16, ConstantPoolInfo> = BTreeMap::new();

        for (old_cp_index, entry) in cp.entries.iter() {
            let new_cp_index: u16 = cp_index_map.get(*old_cp_index);
            new_cp_entries.insert(
                new_cp_index,
                match entry {
                    ConstantPoolInfo::Utf8 { .. }
                    | ConstantPoolInfo::Integer { .. }
                    | ConstantPoolInfo::Float { .. }
                    | ConstantPoolInfo::Long { .. }
                    | ConstantPoolInfo::Double { .. } => entry.clone(),
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
                    ConstantPoolInfo::MethodType { descriptor_index } => {
                        ConstantPoolInfo::MethodType {
                            descriptor_index: cp_index_map.get(*descriptor_index),
                        }
                    }
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
                },
            );
        }
        ConstantPool {
            entries: new_cp_entries,
        }
    }

    fn modify_fields(&self, cp_index_map: &CPIndexMap, fields: &Vec<FieldInfo>) -> Vec<FieldInfo> {
        let mut new_fields = Vec::with_capacity(fields.len());
        for field in fields {
            new_fields.push(FieldInfo {
                access_flags: field.access_flags,
                name_index: cp_index_map.get(field.name_index),
                descriptor_index: cp_index_map.get(field.descriptor_index),
                attributes: self.modify_attributes(cp_index_map, &field.attributes),
            });
        }
        new_fields
    }

    fn modify_methods(
        &self,
        cp_index_map: &CPIndexMap,
        methods: &Vec<MethodInfo>,
    ) -> Vec<MethodInfo> {
        let mut new_methods = Vec::with_capacity(methods.len());
        for method in methods {
            new_methods.push(MethodInfo {
                access_flags: method.access_flags,
                name_index: cp_index_map.get(method.name_index),
                descriptor_index: cp_index_map.get(method.descriptor_index),
                attributes: self.modify_attributes(cp_index_map, &method.attributes),
            });
        }
        new_methods
    }

    fn modify_attributes(
        &self,
        cp_index_map: &CPIndexMap,
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
                    attributes: self.modify_attributes(cp_index_map, attributes),
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
        old_code: &[(u32, BytecodeInstruction)],
    ) -> Vec<(u32, BytecodeInstruction)> {
        let mut new_code: Vec<(u32, BytecodeInstruction)> = Vec::with_capacity(old_code.len());
        for (pos, inst) in old_code.iter() {
            new_code.push((
                *pos,
                match inst {
                    BytecodeInstruction::Ldc {
                        constant_pool_index,
                    } => {
                        let new_index=cp_index_map
                                .get(*constant_pool_index as u16);
                        BytecodeInstruction::Ldc {
                            constant_pool_index: new_index
                                // this conversion is guaranteed to work from shuffle_indices
                                .try_into()
                                .unwrap_or_else(|e| panic!("Could not convert constant pool index {new_index} into u8: {e}.")),
                        }
                    }
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
        *self.map.get(&old_cp_index).unwrap_or_else(|| {
            panic!("Could not retrieve new CP index for old index {old_cp_index}.");
        })
    }
}

impl ClassFileTransformation for ShuffleConstantPool {
    fn transform(&self, cf: &ClassFile) -> ClassFile {
        let special_indices: HashSet<u16> = collect_special_indices(cf);

        let cp_index_map: CPIndexMap = shuffle_indices(
            self.seed,
            &cf.constant_pool
                .entries
                .keys()
                .cloned()
                .collect::<HashSet<u16>>(),
            &special_indices,
        );

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
        let new_fields = self.modify_fields(&cp_index_map, &cf.fields);
        let new_methods = self.modify_methods(&cp_index_map, &cf.methods);
        let new_attributes = self.modify_attributes(&cp_index_map, &cf.attributes);

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

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    #[test]
    fn shuffle_constant_pool_indices() {
        let test_cases: Vec<(HashSet<u16>, HashSet<u16>)> =
            vec![([1, 2, 3].into_iter().collect(), [].into_iter().collect())];

        for (indices, special_indices) in test_cases {
            let seed: u64 = rand::rng().next_u64();
            let new_indices = shuffle_indices(seed, &indices, &special_indices).map;

            assert!(
                indices.len() == new_indices.len(),
                "Call to shuffle_indices() with seed=0x{seed:016x} returned a different number of indices: expected {} but was {}.",
                indices.len(),
                new_indices.len()
            );
            for old_idx in indices {
                assert!(
                    new_indices.contains_key(&old_idx),
                    "Call to shuffle_indices() with seed=0x{seed:016x} returned a map in which the index {old_idx} is not present.",
                );
            }
            for special_idx in special_indices {
                let new_idx = new_indices.get(&special_idx).unwrap();
                assert!(
                    special_idx < 256,
                    "Call to shuffle_indices() with seed=0x{seed:016x} returned a map in which the special index {special_idx} maps to {new_idx}, which cannot be a special index.",
                );
            }
        }
    }
}
