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
