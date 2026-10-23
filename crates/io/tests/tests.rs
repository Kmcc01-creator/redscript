use redscript_io::{
    Class, ClassFlags, Definition, Enum, EnumMember, Field, FieldFlags, Function, FunctionFlags,
    Local, LocalFlags, Parameter, ParameterFlags, ScriptBundle, Type, TypeKind, Visibility,
};

#[test]
fn roundtrip_def() {
    let mut bundle = ScriptBundle::default();
    let name = bundle.cnames_mut().add("whatever");
    let typ = Type::new(name, TypeKind::Class);
    let idx = bundle.define(typ.clone());

    assert_eq!(bundle[idx], typ);
}

#[test]
fn roundtrip_encode() {
    let mut bundle = ScriptBundle::default();
    let name = bundle.cnames_mut().add("whatever");
    let typ = Type::new(name, TypeKind::Class);
    bundle.define(typ.clone());

    let bytes = bundle.into_writeable().to_bytes().unwrap();
    let bundle = ScriptBundle::from_bytes(&bytes).unwrap();
    let defs = bundle.definitions().cloned().collect::<Vec<_>>();
    assert_eq!(&defs, &[Definition::Type(typ)]);
}

const HEADER_SIZE: usize = 104;
const FLAGS_HEADER_SIZE: usize = 116;
const DEFINITION_FLAGS_ENTRY: usize = 12;
// string data, cnames, tweakdb ids, resources, definitions, strings, definition flags
const ENTRY_SIZES: [usize; 7] = [1, 4, 4, 4, 20, 4, 12];

fn read_u32(bytes: &[u8], pos: usize) -> u32 {
    u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap())
}

fn write_u32(bytes: &mut [u8], pos: usize, val: u32) {
    bytes[pos..pos + 4].copy_from_slice(&val.to_le_bytes());
}

fn table(bytes: &[u8], index: usize) -> (usize, usize, u32) {
    let pos = 32 + 12 * index;
    (
        read_u32(bytes, pos) as usize,
        read_u32(bytes, pos + 4) as usize,
        read_u32(bytes, pos + 8),
    )
}

fn header_crc(bytes: &[u8], size: usize) -> u32 {
    let mut header = bytes[..size].to_vec();
    write_u32(&mut header, 24, 0xDEAD_BEEF);
    crc32fast::hash(&header)
}

/// Turns a bundle without definition flags into one with them, laid out as game 2.5's: a
/// 116-byte header, and `entries` as the seventh table after the six others.
fn with_definition_flags(bytes: &[u8], entries: &[[u8; DEFINITION_FLAGS_ENTRY]]) -> Vec<u8> {
    let shift = FLAGS_HEADER_SIZE - HEADER_SIZE;
    let mut out = bytes[..HEADER_SIZE].to_vec();
    out.extend_from_slice(&[0; FLAGS_HEADER_SIZE - HEADER_SIZE]);
    out.extend_from_slice(&bytes[HEADER_SIZE..]);

    let flags = read_u32(&out, 8);
    write_u32(&mut out, 8, flags | 2);
    for index in 0..6 {
        let pos = 32 + 12 * index;
        let offset = read_u32(&out, pos);
        write_u32(&mut out, pos, offset + shift as u32);
    }
    let (defs_offset, defs_count, _) = table(&out, 4);
    for index in 0..defs_count {
        let pos = defs_offset + 20 * index + 8;
        let offset = read_u32(&out, pos);
        if offset != 0 {
            write_u32(&mut out, pos, offset + shift as u32);
        }
    }
    let defs_hash = crc32fast::hash(&out[defs_offset..defs_offset + 20 * defs_count]);
    write_u32(&mut out, 32 + 12 * 4 + 8, defs_hash);

    let flags_offset = out.len();
    let flags = entries.concat();
    out.extend_from_slice(&flags);
    write_u32(&mut out, HEADER_SIZE, flags_offset as u32);
    write_u32(&mut out, HEADER_SIZE + 4, entries.len() as u32);
    write_u32(&mut out, HEADER_SIZE + 8, crc32fast::hash(&flags));

    let crc = header_crc(&out, FLAGS_HEADER_SIZE);
    write_u32(&mut out, 24, crc);
    out
}

/// Checks the header's and every table's checksum, and returns the definition flags, if any.
fn check_bundle(bytes: &[u8]) -> Option<Vec<[u8; DEFINITION_FLAGS_ENTRY]>> {
    let has_flags = read_u32(bytes, 8) & 2 != 0;
    let (header_size, tables) = if has_flags {
        (FLAGS_HEADER_SIZE, 7)
    } else {
        (HEADER_SIZE, 6)
    };
    assert_eq!(read_u32(bytes, 24), header_crc(bytes, header_size));
    assert_eq!(table(bytes, 0).0, header_size);
    for (index, size) in ENTRY_SIZES.iter().enumerate().take(tables) {
        let (offset, count, hash) = table(bytes, index);
        assert_eq!(crc32fast::hash(&bytes[offset..offset + count * size]), hash);
    }
    has_flags.then(|| {
        let (offset, count, _) = table(bytes, 6);
        bytes[offset..offset + count * DEFINITION_FLAGS_ENTRY]
            .chunks_exact(DEFINITION_FLAGS_ENTRY)
            .map(|entry| entry.try_into().unwrap())
            .collect()
    })
}

fn entry(byte0: u8, rest: u8) -> [u8; DEFINITION_FLAGS_ENTRY] {
    let mut entry = [rest; DEFINITION_FLAGS_ENTRY];
    entry[0] = byte0;
    entry
}

#[test]
fn roundtrip_without_definition_flags() {
    let mut bundle = ScriptBundle::default();
    let name = bundle.cnames_mut().add("whatever");
    bundle.define(Type::new(name, TypeKind::Class));

    let bytes = bundle.into_writeable().to_bytes().unwrap();
    assert_eq!(check_bundle(&bytes), None);
    let bytes = ScriptBundle::from_bytes(&bytes)
        .unwrap()
        .into_writeable()
        .to_bytes()
        .unwrap();
    assert_eq!(check_bundle(&bytes), None);
}

#[test]
fn roundtrip_definition_flags() {
    let mut bundle = ScriptBundle::default();
    let name = bundle.cnames_mut().add("whatever");
    let typ = Type::new(name, TypeKind::Class);
    bundle.define(typ.clone());
    let enum_ = bundle.define(Enum::new(name, Visibility::Public, 4));
    bundle.define(EnumMember::new(name, enum_, 7));

    let entries = [
        entry(0, 0),
        entry(0x10, 0xCD),
        entry(0xFC, 0x7F),
        entry(0x2A, 0x23),
    ];
    let input = with_definition_flags(&bundle.into_writeable().to_bytes().unwrap(), &entries);
    assert_eq!(check_bundle(&input).as_deref(), Some(&entries[..]));

    let bundle = ScriptBundle::from_bytes(&input).unwrap();
    assert_eq!(bundle.definitions().next(), Some(&Definition::Type(typ)));
    let output = bundle.into_writeable().to_bytes().unwrap();
    assert_eq!(check_bundle(&output).as_deref(), Some(&entries[..]));
}

#[test]
fn definition_flags_for_added_definitions() {
    let mut bundle = ScriptBundle::default();
    let name = bundle.cnames_mut().add("whatever");
    let typ = bundle.define(Type::new(name, TypeKind::Class));
    let input = with_definition_flags(
        &bundle.into_writeable().to_bytes().unwrap(),
        &[entry(0, 0), entry(0x10, 0xCD)],
    );

    let mut bundle = ScriptBundle::from_bytes(&input).unwrap();
    let class = bundle.define(Class::new(name, Visibility::Public, ClassFlags::new()));
    let method = bundle.define(
        Function::new(name, Visibility::Public, FunctionFlags::new()).with_class(Some(class)),
    );
    bundle.define(Parameter::new(name, method, typ, ParameterFlags::new()));
    bundle.define(Local::new(name, method, typ, LocalFlags::new()));
    bundle.define(Field::new(
        name,
        class,
        Visibility::Public,
        typ,
        FieldFlags::new(),
    ));
    bundle.define(Function::new(
        name,
        Visibility::Public,
        FunctionFlags::new(),
    ));
    let enum_ = bundle.define(Enum::new(name, Visibility::Public, 4));
    bundle.define(EnumMember::new(name, enum_, 7));

    let output = bundle.into_writeable().to_bytes().unwrap();
    let member = entry(2, 0);
    let other = entry(0, 0);
    assert_eq!(
        check_bundle(&output).as_deref(),
        Some(
            &[
                entry(0, 0),
                entry(0x10, 0xCD),
                other,  // class
                member, // method
                other,  // parameter
                other,  // local
                member, // field
                other,  // global function
                other,  // enum
                member, // enum member
            ][..]
        )
    );
}
