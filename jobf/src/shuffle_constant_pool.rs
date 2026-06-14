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

enum CPEntryType {
    Small, // entries which occupy 1 slot
    Big,   // entries which occupy 2 slots (Long and Double)
}

impl CPEntryType {
    fn size(&self) -> u16 {
        match self {
            CPEntryType::Small => 1,
            CPEntryType::Big => 2,
        }
    }
}

struct CPEntry {
    entry_type: CPEntryType,
    is_special: bool,
}

fn shuffle_indices(seed: u64, entries: &Vec<CPEntry>) -> CPIndexMap {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    // Work with a mutable list of (original_index, CPEntry) pairs.
    // CP indices are 1-based, so entry at position i in the vec has original index i+1.
    let mut indexed: Vec<(u16, &CPEntry)> = Vec::with_capacity(entries.len());
    indexed.push((1u16, &entries[0]));
    for i in 1..entries.len() {
        indexed.push((
            indexed[indexed.len() - 1].0 + (indexed[indexed.len() - 1].1.entry_type.size()),
            &entries[i],
        ));
    }
    // entries
    //     .iter()
    //     .enumerate()
    //     .map(|(i, e)| ((i + 1) as u16, e))
    //     .collect();

    let n = indexed.len();

    // Fisher-Yates shuffle, respecting the constraint that special entries
    // must land at positions 0..255 (i.e. new 1-based index <= 255, fitting in a u8).
    //
    // Strategy: when placing position i, if the entry is special we pick a swap
    // target j in [i, 255]; otherwise we pick j in [i, n-1], skipping special
    // entries when i >= 256 to avoid pushing them out of range.
    for i in 0..n.saturating_sub(1) {
        let j = if indexed[i].1.is_special {
            // Must land at a 1-based index <= 255, i.e. 0-based position <= 254.
            // Pick from [i, 254], ensuring we don't go out of bounds.
            let hi = 254usize.min(n - 1);
            assert!(
                i <= hi,
                "Too many special entries to fit within the first 255 CP slots."
            );
            let mut candidate = rng.random_range(i..=hi);
            // Skip other special entries already in the safe zone so we don't
            // waste safe slots — accept the first non-special at or after candidate.
            while candidate < n && candidate != i && indexed[candidate].1.is_special {
                candidate += 1;
                if candidate > hi {
                    candidate = i; // fallback: keep in place
                    break;
                }
            }
            candidate
        } else {
            // Non-special: can go anywhere from i onward, but must not displace
            // a special entry that hasn't been placed yet into a slot >= 255.
            let mut candidate = rng.random_range(i..n);
            // If we're at position >= 255 and we'd pick up a special entry, keep
            // searching until we find a non-special one.
            if i >= 255 {
                let mut attempts = 0;
                while indexed[candidate].1.is_special {
                    candidate = rng.random_range(i..n);
                    attempts += 1;
                    assert!(
                        attempts < n * 4,
                        "Could not find a non-special entry to place at position {i}."
                    );
                }
            }
            candidate
        };
        indexed.swap(i, j);
    }

    // Build the index map: old CP index -> new CP index (1-based position after shuffle).
    let map: HashMap<u16, u16> = indexed
        .iter()
        .enumerate()
        .map(|(new_pos, (old_idx, _))| (*old_idx, (new_pos + 1) as u16))
        .collect();

    CPIndexMap { map }
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

        let entries: Vec<CPEntry> = cf
            .constant_pool
            .entries
            .iter()
            .map(|(idx, info)| CPEntry {
                entry_type: match info {
                    ConstantPoolInfo::Long { .. } | ConstantPoolInfo::Double { .. } => {
                        CPEntryType::Big
                    }
                    _ => CPEntryType::Small,
                },
                is_special: special_indices.contains(idx),
            })
            .collect();

        let cp_index_map = shuffle_indices(self.seed, &entries);

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
    use super::*;
    use rand::Rng;
    use rstest::rstest;

    fn make_entries(
        count: usize,
        special_positions: &HashSet<usize>,
        big_entries: &HashSet<usize>,
    ) -> Vec<CPEntry> {
        (0..count)
            .map(|i| CPEntry {
                entry_type: if big_entries.contains(&i) {
                    CPEntryType::Big
                } else {
                    CPEntryType::Small
                },
                is_special: special_positions.contains(&i),
            })
            .collect()
    }

    #[rstest]
    #[case(0, HashSet::new(), HashSet::new())]
    #[case(5, HashSet::new(), HashSet::new())]
    #[case(1000, [0, 2].into_iter().collect(), HashSet::new())]
    #[case(1000, [999].into_iter().collect(), HashSet::new())]
    #[case(5, HashSet::new(), [2].into_iter().collect())]
    #[case(5, HashSet::new(), [2, 4].into_iter().collect())]
    #[case(1000, [500, 999].into_iter().collect(), [400].into_iter().collect())]
    #[case(1000, [500].into_iter().collect(), [400, 999].into_iter().collect())]
    #[case(1000, [500, 999].into_iter().collect(), [400, 999].into_iter().collect())]
    fn shuffle_constant_pool_indices(
        #[case] count: usize,
        #[case] special_positions: HashSet<usize>,
        #[case] big_entries: HashSet<usize>,
    ) {
        let entries = make_entries(count, &special_positions, &big_entries);

        let seed: u64 = rand::rng().next_u64();
        let result = shuffle_indices(seed, &entries).map;

        assert_eq!(
            result.len(),
            count,
            "seed=0x{seed:016x}: expected {count} mapped indices but were {}.",
            result.len()
        );

        // Every old index must be present as a key.
        for old_idx in 1..=(count as u16) {
            assert!(
                result.contains_key(&old_idx),
                "seed=0x{seed:016x}: old index {old_idx} missing from map."
            );
        }

        // The new indices must be a permutation of 1..=count (bijection).
        let mut new_indices: Vec<u16> = result.values().cloned().collect();
        new_indices.sort_unstable();
        let expected: Vec<u16> = (1..=(count as u16)).collect();
        assert_eq!(
            new_indices, expected,
            "seed=0x{seed:016x}: new indices are not a permutation of 1..={count}."
        );

        // Special entries must map to a new index that fits in a u8 (<= 255).
        for special_pos in &special_positions {
            let old_idx = (*special_pos + 1) as u16; // convert 0-based pos to 1-based CP index
            let new_idx = result[&old_idx];
            assert!(
                new_idx <= 255,
                "seed=0x{seed:016x}: special old index {old_idx} mapped to {new_idx}, which does not fit in u8."
            );
        }
    }
}
