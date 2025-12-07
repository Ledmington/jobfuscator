use std::env;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};

pub enum Endian {
    Little,
    Big,
}

pub struct BinaryReader<'a> {
    buf: &'a [u8],
    pos: usize,
    endian: Endian,
}

impl<'a> BinaryReader<'a> {
    pub fn new(buf: &'a [u8], endian: Endian) -> Self {
        Self {
            buf,
            pos: 0,
            endian,
        }
    }

    fn read_bytes(&mut self, count: usize) -> io::Result<&'a [u8]> {
        debug_assert!(count > 0);
        if self.pos + count > self.buf.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes",
            ));
        }
        let slice = &self.buf[self.pos..self.pos + count];
        self.pos += count;
        Ok(slice)
    }

    pub fn read_u8(&mut self) -> io::Result<u8> {
        Ok(self.read_bytes(1)?[0])
    }

    pub fn read_u16(&mut self) -> io::Result<u16> {
        let bytes: [u8; 2] = self.read_bytes(2).unwrap().try_into().unwrap();
        Ok(match self.endian {
            Endian::Little => u16::from_le_bytes(bytes),
            Endian::Big => u16::from_be_bytes(bytes),
        })
    }

    pub fn read_u16_vec(&mut self, count: usize) -> io::Result<Vec<u16>> {
        debug_assert!(count > 0);
        let mut res: Vec<u16> = vec![0u16; count];
        for x in res.iter_mut().take(count) {
            *x = self.read_u16().unwrap();
        }
        Ok(res)
    }

    pub fn read_u32(&mut self) -> io::Result<u32> {
        let bytes: [u8; 4] = self.read_bytes(4).unwrap().try_into().unwrap();
        Ok(match self.endian {
            Endian::Little => u32::from_le_bytes(bytes),
            Endian::Big => u32::from_be_bytes(bytes),
        })
    }
}

/**
 * Specification available at https://docs.oracle.com/javase/specs/jvms/se25/html/jvms-4.html
 */
struct ClassFile {
    absolute_file_path: String,
    minor_version: u16,
    major_version: u16,
    constant_pool: Vec<ConstantPoolInfo>,
    access_flags: u16,
    this_class: u16,
    super_class: u16,
    interfaces: Vec<u16>,
    fields: Vec<FieldInfo>,
    methods: Vec<MethodInfo>,
    attributes: Vec<AttributeInfo>,
}

struct ConstantPoolInfo {
    tag: u8,
    info: Vec<u8>,
}

struct FieldInfo {}

struct MethodInfo {}

struct AttributeInfo {}

fn absolute_no_symlinks(p: &Path) -> std::io::Result<PathBuf> {
    if p.is_absolute() {
        Ok(p.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(p))
    }
}

fn parse_class_file(filename: String) -> ClassFile {
    let abs_file_path = absolute_no_symlinks(Path::new(&filename)).unwrap();
    let file = File::open(&abs_file_path).expect("File does not exist");
    let mut file_reader = BufReader::new(file);
    let mut file_bytes: Vec<u8> = Vec::with_capacity(file_reader.capacity());
    file_reader
        .read_to_end(&mut file_bytes)
        .expect("Could not read whole file");

    let mut reader = BinaryReader::new(&file_bytes, Endian::Big);

    let actual_magic_number: u32 = reader.read_u32().unwrap();
    const EXPECTED_MAGIC_NUMBER: u32 = 0xcafebabe;
    if actual_magic_number != EXPECTED_MAGIC_NUMBER {
        panic!(
            "Wrong magic number: expected 0x{:08x} but was 0x{:08x}.",
            EXPECTED_MAGIC_NUMBER, actual_magic_number
        );
    }

    let minor_version: u16 = reader.read_u16().unwrap();
    let major_version: u16 = reader.read_u16().unwrap();

    let cp_count: u16 = reader.read_u16().unwrap();
    let cp: Vec<ConstantPoolInfo> = Vec::with_capacity((cp_count - 1).into());

    let flags: u16 = reader.read_u16().unwrap();

    let this_class: u16 = reader.read_u16().unwrap();
    let super_class: u16 = reader.read_u16().unwrap();

    let interfaces_count: u16 = reader.read_u16().unwrap();
    let interfaces: Vec<u16> = reader.read_u16_vec(interfaces_count.into()).unwrap();

    let fields_count: u16 = reader.read_u16().unwrap();
    let fields: Vec<FieldInfo> = Vec::with_capacity(fields_count.into());

    let methods_count: u16 = reader.read_u16().unwrap();
    let methods: Vec<MethodInfo> = Vec::with_capacity(methods_count.into());

    let attributes_count: u16 = reader.read_u16().unwrap();
    let attributes: Vec<AttributeInfo> = Vec::with_capacity(attributes_count.into());

    ClassFile {
        absolute_file_path: abs_file_path.to_str().unwrap().to_string(),
        minor_version,
        major_version,
        constant_pool: cp,
        access_flags: flags,
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        attributes,
    }
}

fn print_class_file(classfile: &ClassFile) {
    println!("Classfile {}", classfile.absolute_file_path);
    println!("  minor version: {}", classfile.minor_version);
    println!("  major version: {}", classfile.major_version);
    println!("  flags: 0x{:04x}", classfile.access_flags);
    println!("  this class: #{}", classfile.this_class);
    println!("  super class: #{}", classfile.super_class);
    println!(
        " interfaces: {}, fields: {}, methods: {}, attributes: {}",
        classfile.interfaces.len(),
        classfile.fields.len(),
        classfile.methods.len(),
        classfile.attributes.len()
    );
    println!("Constant pool:");
}

fn main() -> io::Result<()> {
    let filename = env::args().nth(1).expect("Usage: program <filename>");

    let classfile: ClassFile = parse_class_file(filename);

    print_class_file(&classfile);

    Ok(())
}
