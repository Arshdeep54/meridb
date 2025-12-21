use crc32fast::Hasher;

use crate::error::{CatalogError, Result};

const MAGIC: [u8; 4] = *b"MDB0";
const VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct MetaDecoded {
    pub name: String,
    pub created_at: u64,
    pub version: u32,
    pub tables_count: u32,
}

pub fn encode_meta(name: &str, created_at: u64, tables_count: u32) -> Vec<u8> {
    // Layout (LE endianness):
    // magic[4] | version(u32) | created_at(u64) | name_len(u16) | name bytes | tables_count(u32)
    // | reserved(u32=0) | checksum(u32=CRC32(all previous bytes))
    let mut buf = Vec::with_capacity(4 + 4 + 8 + 2 + name.len() + 4 + 4 + 4);

    buf.extend_from_slice(&MAGIC);
    buf.extend_from_slice(&VERSION.to_le_bytes());
    buf.extend_from_slice(&created_at.to_le_bytes());

    let name_bytes = name.as_bytes();
    let name_len = u16::try_from(name_bytes.len()).unwrap_or(u16::MAX);
    buf.extend_from_slice(&name_len.to_le_bytes());
    buf.extend_from_slice(name_bytes);

    buf.extend_from_slice(&tables_count.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes()); // reserved

    let mut hasher = Hasher::new();
    hasher.update(&buf);
    let checksum = hasher.finalize();
    buf.extend_from_slice(&checksum.to_le_bytes());

    buf
}

pub fn decode_meta(bytes: &[u8]) -> Result<MetaDecoded> {
    let need_min = 4 + 4 + 8 + 2 + 4 + 4 + 4;
    if bytes.len() < need_min {
        return Err(CatalogError::MetaTooShort {
            min: need_min,
            actual: bytes.len(),
        });
    }

    let (magic, rest) = bytes.split_at(4);
    if magic != MAGIC {
        return Err(CatalogError::BadMagic);
    }

    let (ver_b, rest) = rest.split_at(4);
    let version = u32::from_le_bytes(ver_b.try_into().unwrap());
    if version == 0 {
        return Err(CatalogError::BadVersion { version });
    }

    let (created_b, rest) = rest.split_at(8);
    let created_at = u64::from_le_bytes(created_b.try_into().unwrap());

    let (name_len_b, rest) = rest.split_at(2);
    let name_len = u16::from_le_bytes(name_len_b.try_into().unwrap()) as usize;

    if rest.len() < name_len + 4 + 4 + 4 {
        return Err(CatalogError::Truncated);
    }
    let (name_b, rest) = rest.split_at(name_len);
    let name = std::str::from_utf8(name_b)
        .map_err(|_| CatalogError::BadUtf8)?
        .to_string();

    let (tables_b, rest) = rest.split_at(4);
    let tables_count = u32::from_le_bytes(tables_b.try_into().unwrap());

    let (reserved_b, rest) = rest.split_at(4);
    let _reserved = u32::from_le_bytes(reserved_b.try_into().unwrap());

    let (checksum_b, _rest) = rest.split_at(4);
    let checksum = u32::from_le_bytes(checksum_b.try_into().unwrap());

    // Verify CRC32 over everything before checksum
    let mut hasher = Hasher::new();
    hasher.update(&bytes[..bytes.len() - 4]);
    let expect = hasher.finalize();
    if expect != checksum {
        return Err(CatalogError::ChecksumMismatch {
            expected: expect,
            got: checksum,
        });
    }

    Ok(MetaDecoded {
        name,
        created_at,
        version,
        tables_count,
    })
}
