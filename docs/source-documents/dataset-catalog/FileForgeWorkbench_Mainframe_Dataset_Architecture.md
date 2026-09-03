# FileForgeWorkbench Mainframe Dataset Architecture

## 1. Purpose

This document defines the storage architecture for FileForgeWorkbench mainframe dataset emulation.

The goal is to:

- Preserve authentic z/OS dataset behaviour.
- Support PS, PDS, PDSE, GDG, VSAM and POSIX files.
- Remain cross-platform.
- Avoid unnecessary dependence on proprietary formats.
- Support Git integration.
- Support future JES/CICS/Batch simulation.

---

# 2. Architectural Principles

## Principle 1: Separate Metadata From Data

Metadata SHALL be stored in SQLite.

Dataset contents SHALL be stored using the most appropriate physical implementation.

## Principle 2: Preserve Mainframe Record Semantics

Mainframe datasets SHALL remain record-oriented.

Dataset records SHALL NOT be represented internally using:

- CRLF
- LF
- text file line terminators

Record boundaries shall be derived from:

- RECFM
- LRECL
- RDW
- VSAM key structure

## Principle 3: POSIX Files Remain Native

POSIX files SHALL remain native operating system files.

FileForgeWorkbench SHALL NOT attempt to convert POSIX files into mainframe dataset structures.

---

# 3. Catalogue Architecture

SQLite acts as the master catalogue.

```sql
DATASET
(
   DATASET_ID,
   DSNAME,
   DSORG,
   RECFM,
   LRECL,
   BLKSIZE,
   PHYSICAL_PATH,
   CREATED_DATE,
   MODIFIED_DATE
)
```

---

# 4. Physical Storage Strategy

| Dataset Type | Physical Storage | Metadata Storage |
|-------------|-----------------|------------------|
| PS | Binary dataset file | SQLite |
| PDS | Folder with member files | SQLite |
| PDSE | Folder with member files | SQLite |
| GDG | Binary dataset files | SQLite |
| VSAM KSDS | SQLite Tables | SQLite |
| VSAM RRDS | SQLite Tables | SQLite |
| VSAM ESDS | Binary dataset files | SQLite |
| POSIX | Native filesystem | SQLite catalog reference |

---

# 5. Record Formats

## Fixed-Length (F)

Example:

```text
RECFM=F
LRECL=80
```

Storage:

```text
[80 bytes][80 bytes][80 bytes]
```

No CRLF or LF terminators exist.

Record position:

```text
Offset = RecordNumber × LRECL
```

## Fixed Blocked (FB)

Example:

```text
RECFM=FB
LRECL=80
BLKSIZE=8000
```

Storage:

```text
[80][80][80][80][80]...
```

FileForgeWorkbench stores logical records only.

BLKSIZE is retained as metadata for:

- JCL validation
- Performance modelling
- Mainframe emulation fidelity

Physical blocks are not persisted separately.

## Variable-Length (V)

Each record contains an RDW (Record Descriptor Word).

```text
+------+-------------+
| RDW  | DATA        |
+------+-------------+
```

Storage:

```text
[RDW][DATA]
[RDW][DATA]
[RDW][DATA]
```

No CRLF or LF delimiters are used.

## Variable Blocked (VB)

Example:

```text
RECFM=VB
```

Storage:

```text
[RDW][DATA]
[RDW][DATA]
[RDW][DATA]
```

The workstation preserves RDW semantics.

Blocking characteristics remain metadata attributes.

---

# 6. PDS and PDSE Libraries

Example:

```text
datasets/
   PROD.JCL/
      JOB1.mem
      JOB2.mem
      JOB3.mem
```

Members remain record-oriented binary files.

Even COBOL, JCL, CLIST and COPYBOOK members retain their original RECFM/LRECL definitions.

The editor displays records as lines while preserving binary dataset representation internally.

---

# 7. GDG Architecture

Physical datasets:

```text
PAYROLL.EXTRACT.G0001V00.ds
PAYROLL.EXTRACT.G0002V00.ds
PAYROLL.EXTRACT.G0003V00.ds
```

Generation relationships are maintained in SQLite.

Logical references:

```text
PAYROLL.EXTRACT(+1)
PAYROLL.EXTRACT(0)
PAYROLL.EXTRACT(-1)
```

are resolved through the catalogue.

---

# 8. VSAM Architecture

## KSDS

Implemented using SQLite tables.

```sql
CUSTOMER_MASTER
(
   VSAM_KEY TEXT PRIMARY KEY,
   RECORD_DATA BLOB
)
```

## RRDS

```sql
CUSTOMER_FILE
(
   RECNO INTEGER PRIMARY KEY,
   RECORD_DATA BLOB
)
```

## ESDS

Stored as append-only binary dataset files.

---

# 9. POSIX Files

POSIX files remain native operating-system files.

Examples:

```text
/home/alan/test.txt
C:\Projects\example.txt
```

Supported native formats include:

- LF text files
- CRLF text files
- UTF-8
- UTF-16
- Binary files

FileForgeWorkbench does not convert or reinterpret native filesystem semantics.

---

# 10. Dataset Editor Design

The editor operates on records rather than byte streams.

Example display:

```text
000001 //JOB1 JOB ...
000002 //STEP1 EXEC ...
000003 //SYSIN DD *
```

Internally:

```text
80-byte records
```

User experience:

```text
Line-oriented editing
```

Storage representation:

```text
Record-oriented binary datasets
```

---

# 11. Recommended Rust Abstraction

```rust
trait DatasetProvider {
    fn open();
    fn read_record();
    fn write_record();
    fn insert_record();
    fn delete_record();
    fn get_attributes();
}
```

Implementations:

```text
SequentialDatasetProvider
PDSMemberProvider
GDGDatasetProvider
VSAMKSDSProvider
VSAMRRDSProvider
VSAMESDSProvider
POSIXFileProvider
```

---

# 12. Key Design Decision

**FileForgeWorkbench SHALL treat mainframe datasets as record-oriented entities, not text files.**

Record boundaries SHALL be determined by:

- RECFM
- LRECL
- RDW
- VSAM structures

and never by:

- LF
- CRLF
- platform-specific text delimiters

This preserves authentic z/OS dataset semantics while remaining portable across Windows, Linux and macOS.
