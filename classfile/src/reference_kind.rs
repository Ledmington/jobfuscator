#![forbid(unsafe_code)]

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ReferenceKind {
    GetField = 1,
    GetStatic = 2,
    PutField = 3,
    PutStatic = 4,
    InvokeVirtual = 5,
    InvokeStatic = 6,
    InvokeSpecial = 7,
    NewInvokeSpecial = 8,
    InvokeInterface = 9,
}

impl From<u8> for ReferenceKind {
    fn from(value: u8) -> Self {
        match value {
            1 => ReferenceKind::GetField,
            2 => ReferenceKind::GetStatic,
            3 => ReferenceKind::PutField,
            4 => ReferenceKind::PutStatic,
            5 => ReferenceKind::InvokeVirtual,
            6 => ReferenceKind::InvokeStatic,
            7 => ReferenceKind::InvokeSpecial,
            8 => ReferenceKind::NewInvokeSpecial,
            9 => ReferenceKind::InvokeInterface,
            _ => panic!("Unknwon reference_kind value {}.", value),
        }
    }
}

// TODO: Convert into a trait?
pub fn java_repr(ref_kind: ReferenceKind) -> String {
    match ref_kind {
        ReferenceKind::GetField => "REF_getField",
        ReferenceKind::GetStatic => "REF_GetStatic",
        ReferenceKind::PutField => "REF_PutField",
        ReferenceKind::PutStatic => "REF_PutStatic",
        ReferenceKind::InvokeVirtual => "REF_invokeVirtual",
        ReferenceKind::InvokeStatic => "REF_invokeStatic",
        ReferenceKind::InvokeSpecial => "REF_InvokeSpecial",
        ReferenceKind::NewInvokeSpecial => "REF_NewInvokeSpecial",
        ReferenceKind::InvokeInterface => "REF_InvokeInterface",
    }
    .to_string()
}
