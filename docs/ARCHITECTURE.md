# MeriDB Storage Architecture

This document describes the on-disk and in-memory layout used by the `storage/` crate. It covers heap files, pages, slot directories, record format, and how common operations (INSERT/UPDATE/DELETE/SELECT) work in MeriDB's current design.

- Scope: `crates/storage/`
- Also referenced by: `crates/catalog/` (for file IO and hydration), `crates/exec/` (for planning/execution)

---

## 1. Files, Segments, Pages

- A table is persisted under: `data/<db>/tables/<table>/`
  - `schema.tbl`  — binary schema (name, columns, types, nullable) with CRC.
  - `data/heap.0001` — the first heap segment (more segments later: `heap.0002`, ...).

- A heap segment is a sequence of fixed-size pages. In MeriDB:
  - `PAGE_SIZE = 8192` bytes (8 KiB).
  - Pages are addressed by `page_id: u32` (0-based) within a segment.

```
heap.0001  (8 KiB pages)

+----------+----------+----------+----------+----------+-- ... --+
| page  0  | page  1  | page  2  | page  3  | page  4  |         |
+----------+----------+----------+----------+----------+---------+
^          ^          ^          ^          ^                   ^
0       8192       16384      24576      32768                 EOF
```

---

## 2. Page Layout (Heap Page)

Each page stores multiple tuple payloads and a slot directory at the end (growing backwards). The free space is in the middle, between the payload region and the slot directory.

```
Offset 0                                                     Offset 8192
+------------------- Header -------------------+----------------------+
| Magic "HPG0" (4) | Ver (4) | ... | RecCount |   Payload Region     |
+----------------------------------------------+----------------------+
                                                   ^             ^
                                                   |             |
                                         payload appends   slot directory grows
                                         upward (low->hi)  downward (hi->low)

Slot Directory (at the end of the page):
+----------------------+----------------------+----------------------+  <- end
| Slot[N-1] (5 bytes)  | Slot[N-2] (5 bytes)  | ...                  |
+----------------------+----------------------+----------------------+

Slot entry (5 bytes):
- offset: u16  -> start of payload (relative to page start)
- length: u16  -> payload length in bytes
- flags: u8    -> bit flags (bit 0 = tombstone)
```

- Header fields used:
  - Magic `HPG0` at `[0..4]`, version `u32` at `[4..8]`.
  - `record_count: u16` at `[12..14]` (number of slots).
- Slot length constant: `SLOT_LEN = 5` bytes.
- Header length constant: `HEADER_LEN` (fits fields above; see code).

Free space accounting:
- Find `max_end` of all live payloads (max `offset + length`).
- Slot directory starts at: `slot_dir_start = PAGE_SIZE - record_count * SLOT_LEN`.
- Free bytes (approx): `free = slot_dir_start - max_end`.

---

## 3. Record (Row) Payload Format

Each tuple payload is a concatenation of:

```
[row_id: u64][null_bitmap (ceil(N/8) bytes)][column values ...]
```

- `row_id` (`RowId = u64`) is the stable logical identifier for the row.
- `null_bitmap`: bit `i` set => column `i` is NULL.
- Column values are encoded by type:
  - INTEGER (i64): 8 bytes LE
  - FLOAT (f64): 8 bytes LE
  - BOOLEAN: 1 byte (0/1)
  - TEXT/CHAR/BLOB/JSON: length (u32 LE) + bytes

Example (N=3 columns):
```
+---------+-----------------+-------------------------------+
| row_id  | bitmap (1 byte) | col0 | col1 | col2            |
+---------+-----------------+-------------------------------+
   8 B          1 B           ...   ...    ... variable
```

Deserializer returns `(RowId, Record)`; serializer requires `RowId`.

---

## 4. Slot Directory and Tombstones

- Every logical row has a corresponding slot entry in its page.
- Deleting or superseding a row marks its slot as a tombstone (flags bit 0 = 1).
- SELECT iterators skip tombstoned slots.

```
Slot flags (u8):
- bit 0: 1 => tombstone (dead)
- other bits: reserved
```

---

## 5. Page Editing Primitives

Defined in `storage/page.rs`:
- `iter_slots(buf) -> Iterator<(offset, length, flags)>`
- `page_append(buf, payload) -> Result<slot_id>`
  - Appends payload at end of used payload region.
  - Adds a new slot entry at the end of the slot directory.
  - Updates `record_count`.
- `page_overwrite_if_fits(buf, slot_id, new_payload) -> Result<bool>`
  - Overwrites in-place if `new_payload.len() <= old_length`.
  - Returns `true` if overwritten, `false` if caller must append elsewhere.
- `page_set_tombstone(buf, slot_id)`
  - Marks a slot as dead (tombstone).

ASCII visual for append:
```
Before:
+ header + used payloads + free .................+ slots .....+
                                                 ^            ^
After append:
+ header + used payloads + NEW PAYLOAD + free ..+ new slot ..+
                                                         ^
                                        (directory grows downward)
```

---

## 6. FSM (Free Space Map) and RowIndex

- FSM: in-memory `HashMap<page_id, free_bytes>` per table.
  - Built at `USE <db>` by scanning pages.
  - Updated after each append/overwrite.
  - Used to choose a page for new payloads (`choose_page_for`).

- RowIndex: in-memory `HashMap<RowId, TupleLoc>` per table.
  - `TupleLoc { seg: u32, page_id: u32, slot_id: u16, flags: u8 }`.
  - Built at `USE <db>` by scanning pages/slots and reading `row_id` from payloads.
  - Updated on append/overwrite/tombstone.

These structures live in `FileCatalog` (see `crates/catalog/src/file_catalog.rs`) under `TableState` and are not persisted yet (rebuilt on startup).

---

## 7. Common Operations

### INSERT
1) Build `Record` and assign a new `RowId`.
2) Serialize with `serialize_record_for_page(row_id, &record, columns)`.
3) Pick a page with enough space using FSM.
4) Read page, `page_append`, write page back.
5) Update `RowIndex` with `(row_id -> TupleLoc)` and FSM.

### UPDATE
- Two paths:
  - In-place: If new payload is ≤ old length, `page_overwrite_if_fits` on old page/slot.
  - Append+Tombstone: Append new payload elsewhere, then `page_set_tombstone` old slot.
- Always update RowIndex to point to the new location.

### DELETE
- Lookup old location via RowIndex; `page_set_tombstone` the old slot.
- Remove RowId from RowIndex.

### SELECT
- Sequential page scan (disk): iterate pages, then slots; skip tombstones.
- Reconstruct `(RowId, Record)` via `deserialize_record_for_page`.
- Apply WHERE, projection, print results.
- Optionally use RowIndex to restrict to latest version if multiple versions exist (current RowIndex keeps latest location per RowId).

---

## 8. Example Page With Two Rows

```
Page (8 KiB)

+---------------- header ----------------+--------------------- payloads ----------------------+---- slots ----+
| Magic HPG0 | Ver=1 | ... | RecCount=2 | [row0 payload] [row1 payload] [free ..............] | s0 | s1 |    |
+----------------------------------------+-----------------------------------------------------+----+----+----+
                                                                               ^                     ^    ^
                                                                       free grows left            s0   s1

s0: { off=H+0, len=len(row0), flags=0 }
s1: { off=H+len(row0), len=len(row1), flags=0 }
```

After deleting row0:
```
s0.flags = 1 (tombstone)
```

After updating row1 with larger payload:
```
- Append new row1' payload at end of free.
- Add new slot s2 for row1'.
- Set s1.flags = 1 (tombstone).
- RowIndex[row1] -> (page_id, s2)
```

---
