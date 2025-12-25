use crc32fast::Hasher;
use storage::types::Column;
use types::tokens::DataType;

// Binary layout (LE):
// magic[4] = b"TBL0"
// version u32 = 1
// table_name_len u16
// table_name bytes (UTF-8)
// column_count u16
// for each column:
//   name_len u16
//   name bytes (UTF-8)
//   data_type_code u16   // stable mapping (see data_type_to_code)
//   nullable u8          // 0/1
//   reserved u8          // 0 for now
// table_flags u32        // reserved 0
// checksum u32           // CRC32 of everything before checksum
const MAGIC: [u8; 4] = *b"TBL0";
const VERSION: u32 = 1;

pub fn encode_schema(table_name: &str, columns: &[Column]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(64 + columns.len() * 32);

    buf.extend_from_slice(&MAGIC);
    buf.extend_from_slice(&VERSION.to_le_bytes());

    let table_name_bytes = table_name.as_bytes();
    let table_name_len = u16::try_from(table_name_bytes.len()).unwrap_or(u16::MAX);
    buf.extend_from_slice(&table_name_len.to_le_bytes());
    buf.extend_from_slice(table_name_bytes);

    let col_count: u16 = columns.len().try_into().unwrap_or(u16::MAX);
    buf.extend_from_slice(&col_count.to_le_bytes());

    for col in columns {
        let name_bytes = col.name.as_bytes();
        let name_len = u16::try_from(name_bytes.len()).unwrap_or(u16::MAX);

        buf.extend_from_slice(&name_len.to_le_bytes());
        buf.extend_from_slice(name_bytes);

        let dt_code = data_type_to_code(&col.data_type);
        buf.extend_from_slice(&dt_code.to_le_bytes());

        let nullable = if col.nullable { 1u8 } else { 0u8 };
        buf.push(nullable);

        buf.push(0u8); // reserved per-column
    }

    buf.extend_from_slice(&0u32.to_le_bytes());

    let mut hasher = Hasher::new();
    hasher.update(&buf);
    let checksum = hasher.finalize();
    buf.extend_from_slice(&checksum.to_le_bytes());

    buf
}

fn data_type_to_code(dt: &DataType) -> u16 {
    match dt {
        DataType::INTEGER => 1,
        DataType::FLOAT => 2,
        DataType::TEXT => 3,
        DataType::BOOLEAN => 4,
        DataType::DATE => 5,
        DataType::TIME => 6,
        DataType::TIMESTAMP => 7,
        DataType::DATETIME => 8,
        DataType::CHAR => 9,
        DataType::BLOB => 10,
        DataType::JSON => 11,
        DataType::DECIMAL => 12,
        DataType::DOUBLE => 13,
        DataType::REAL => 14,
        DataType::NUMERIC => 15,
        DataType::TINYINT => 16,
        DataType::SMALLINT => 17,
        DataType::MEDIUMINT => 18,
        DataType::BIGINT => 19,
    }
}

fn data_type_from_code(code: u16) -> Result<DataType, String> {
    match code {
        1 => Ok(DataType::INTEGER),
        2 => Ok(DataType::FLOAT),
        3 => Ok(DataType::TEXT),
        4 => Ok(DataType::BOOLEAN),
        5 => Ok(DataType::DATE),
        6 => Ok(DataType::TIME),
        7 => Ok(DataType::TIMESTAMP),
        8 => Ok(DataType::DATETIME),
        9 => Ok(DataType::CHAR),
        10 => Ok(DataType::BLOB),
        11 => Ok(DataType::JSON),
        12 => Ok(DataType::DECIMAL),
        13 => Ok(DataType::DOUBLE),
        14 => Ok(DataType::REAL),
        15 => Ok(DataType::NUMERIC),
        16 => Ok(DataType::TINYINT),
        17 => Ok(DataType::SMALLINT),
        18 => Ok(DataType::MEDIUMINT),
        19 => Ok(DataType::BIGINT),
        _ => Err(format!("unknown data type code {}", code)),
    }
}

pub fn decode_schema(bytes: &[u8]) -> Result<(String, Vec<Column>), crate::error::CatalogError> {
    use crate::error::CatalogError;

    if bytes.len() < 4 + 4 + 2 + 2 + 4 + 4 {
        return Err(CatalogError::MetaTooShort {
            min: 20,
            actual: bytes.len(),
        });
    }

    let (magic, rest) = bytes.split_at(4);
    if magic != MAGIC {
        return Err(CatalogError::BadMagic);
    }

    let (ver_b, rest) = rest.split_at(4);
    let version = u32::from_le_bytes(ver_b.try_into().unwrap());
    if version != VERSION {
        return Err(CatalogError::BadVersion { version });
    }

    let (tname_len_b, rest) = rest.split_at(2);
    let tname_len = u16::from_le_bytes(tname_len_b.try_into().unwrap()) as usize;

    if rest.len() < tname_len + 2 {
        return Err(CatalogError::Truncated);
    }

    let (tname_b, rest) = rest.split_at(tname_len);
    let table_name = std::str::from_utf8(tname_b)
        .map_err(|_| CatalogError::BadUtf8)?
        .to_string();

    let (col_cnt_b, mut rest) = rest.split_at(2);
    let col_cnt = u16::from_le_bytes(col_cnt_b.try_into().unwrap()) as usize;

    let mut columns = Vec::with_capacity(col_cnt);
    for _ in 0..col_cnt {
        if rest.len() < 2 {
            return Err(CatalogError::Truncated);
        }
        let (name_len_b, r1) = rest.split_at(2);
        let name_len = u16::from_le_bytes(name_len_b.try_into().unwrap()) as usize;
        if r1.len() < name_len + 2 + 1 + 1 {
            return Err(CatalogError::Truncated);
        }
        let (name_b, r2) = r1.split_at(name_len);
        let (code_b, r3) = r2.split_at(2);
        let (nullable_b, r4) = r3.split_at(1);
        let (_reserved_b, r5) = r4.split_at(1);

        let name = std::str::from_utf8(name_b)
            .map_err(|_| CatalogError::BadUtf8)?
            .to_string();
        let code = u16::from_le_bytes(code_b.try_into().unwrap());
        let dt = data_type_from_code(code).map_err(|e| CatalogError::WriteFile {
            path: std::path::PathBuf::from("schema.tbl"),
            source: std::io::Error::other(e),
        })?;
        let nullable = nullable_b[0] != 0;

        columns.push(Column::new(name, dt, nullable));
        rest = r5;
    }

    if rest.len() < 4 + 4 {
        return Err(CatalogError::Truncated);
    }
    let (flags_b, rest) = rest.split_at(4);
    let _flags = u32::from_le_bytes(flags_b.try_into().unwrap());

    let (checksum_b, _tail) = rest.split_at(4);
    let checksum = u32::from_le_bytes(checksum_b.try_into().unwrap());
    let mut hasher = Hasher::new();
    hasher.update(&bytes[..bytes.len() - 4]);
    let expect = hasher.finalize();
    if expect != checksum {
        return Err(CatalogError::ChecksumMismatch {
            expected: expect,
            got: checksum,
        });
    }

    Ok((table_name, columns))
}
