use classfile::{
    access_flags::{
        ClassAccessFlag, ClassAccessFlags, FieldAccessFlag, FieldAccessFlags, InnerClassAccessFlag,
        InnerClassAccessFlags, MethodAccessFlag, MethodAccessFlags,
    },
    attributes::{AttributeInfo, InnerClassInfo},
    classfile::ClassFile,
    fields::FieldInfo,
    methods::MethodInfo,
};

pub(crate) fn make_everything_public(cf: &ClassFile) -> ClassFile {
    ClassFile {
        minor_version: cf.minor_version,
        major_version: cf.major_version,
        constant_pool: cf.constant_pool.clone(),
        access_flags: make_class_flags_public(cf.access_flags),
        this_class: cf.this_class,
        super_class: cf.super_class,
        interfaces: cf.interfaces.clone(),
        fields: make_fields_public(&cf.fields),
        methods: make_methods_public(&cf.methods),
        attributes: make_attributes_public(&cf.attributes),
    }
}

fn make_class_flags_public(flags: ClassAccessFlags) -> ClassAccessFlags {
    ClassAccessFlags::from(flags.to_u16() | (ClassAccessFlag::Public as u16))
}

fn make_fields_public(fields: &[FieldInfo]) -> Vec<FieldInfo> {
    fields.iter().map(make_field_public).collect()
}

fn make_field_public(f: &FieldInfo) -> FieldInfo {
    FieldInfo {
        access_flags: make_field_flags_public(f.access_flags),
        name_index: f.name_index,
        descriptor_index: f.descriptor_index,
        attributes: make_attributes_public(&f.attributes),
    }
}

fn make_field_flags_public(flags: FieldAccessFlags) -> FieldAccessFlags {
    let mut f: u16 = flags.to_u16();
    f |= FieldAccessFlag::Public as u16;
    f &= !(FieldAccessFlag::Private as u16);
    f &= !(FieldAccessFlag::Protected as u16);
    FieldAccessFlags::from(f)
}

fn make_methods_public(methods: &[MethodInfo]) -> Vec<MethodInfo> {
    methods.iter().map(make_method_public).collect()
}

fn make_method_public(m: &MethodInfo) -> MethodInfo {
    MethodInfo {
        access_flags: make_method_flags_public(m.access_flags),
        name_index: m.name_index,
        descriptor_index: m.descriptor_index,
        attributes: make_attributes_public(&m.attributes),
    }
}

fn make_method_flags_public(flags: MethodAccessFlags) -> MethodAccessFlags {
    let mut f: u16 = flags.to_u16();
    f |= MethodAccessFlag::Public as u16;
    f &= !(MethodAccessFlag::Private as u16);
    f &= !(MethodAccessFlag::Protected as u16);
    MethodAccessFlags::from(f)
}

fn make_attributes_public(attributes: &[AttributeInfo]) -> Vec<AttributeInfo> {
    attributes.iter().map(make_attribute_public).collect()
}

fn make_attribute_public(attribute: &AttributeInfo) -> AttributeInfo {
    match attribute {
        AttributeInfo::LineNumberTable { .. }
        | AttributeInfo::LocalVariableTable { .. }
        | AttributeInfo::LocalVariableTypeTable { .. }
        | AttributeInfo::StackMapTable { .. }
        | AttributeInfo::SourceFile { .. }
        | AttributeInfo::BootstrapMethods { .. }
        | AttributeInfo::MethodParameters { .. }
        | AttributeInfo::Record { .. }
        | AttributeInfo::Signature { .. }
        | AttributeInfo::NestMembers { .. }
        | AttributeInfo::RuntimeVisibleAnnotations { .. }
        | AttributeInfo::ConstantValue { .. }
        | AttributeInfo::Exceptions { .. }
        | AttributeInfo::EnclosingMethod { .. } => attribute.clone(),
        AttributeInfo::Code {
            name_index,
            max_stack,
            max_locals,
            code,
            exception_table,
            attributes,
        } => AttributeInfo::Code {
            name_index: *name_index,
            max_stack: *max_stack,
            max_locals: *max_locals,
            code: code.clone(),
            exception_table: exception_table.clone(),
            attributes: make_attributes_public(attributes),
        },
        AttributeInfo::InnerClasses {
            name_index,
            classes,
        } => AttributeInfo::InnerClasses {
            name_index: *name_index,
            classes: classes
                .iter()
                .map(|ic| {
                    let InnerClassInfo {
                        inner_class_info_index,
                        outer_class_info_index,
                        inner_name_index,
                        inner_class_access_flags,
                    } = ic;
                    InnerClassInfo {
                        inner_class_info_index: *inner_class_info_index,
                        outer_class_info_index: *outer_class_info_index,
                        inner_name_index: *inner_name_index,
                        inner_class_access_flags: make_inner_class_flags_public(
                            *inner_class_access_flags,
                        ),
                    }
                })
                .collect(),
        },
    }
}

fn make_inner_class_flags_public(flags: InnerClassAccessFlags) -> InnerClassAccessFlags {
    InnerClassAccessFlags::from(flags.to_u16() | (InnerClassAccessFlag::Public as u16))
}
