# rust_macro_internal

`rust_macro_internal` is an internal procedural macro crate in the `CLR` workspace.

It provides compile-time code generation driven by source artifacts (INI, CSV, JSON, and Mermaid files), and is used as the implementation layer behind higher-level macro workflows.

## Direction

`rust_macro_internal` is moving toward a broader **diagram-to-code** vision: using structured, human-readable diagrams as the authoritative source of truth for generated Rust code.

Rather than writing boilerplate by hand, the goal is to express intent visually — state machines, packet layouts, call flows, data schemas — and have the macro layer synthesize correct, idiomatic Rust from those artifacts at compile time.

This positions the crate as an exploration of what the Rust ecosystem could gain from first-class diagram-driven codegen: fewer hand-maintained structs, tighter coupling between documentation and implementation, and a path toward making complex system designs directly executable.

Mermaid is the current substrate for this work, with INI, CSV, and JSON as supporting formats for data-oriented generation. The longer-term direction is richer diagram semantics — sequence flows becoming async function scaffolds, state diagrams becoming typestate machines, packet diagrams becoming zero-copy layout structs — all driven by the same compile-time macro machinery.

## Location

- Crate path: `CLR/rust_macro_internal`
- Workspace root: `CLR/Cargo.toml`

## Crate type

This crate is a `proc-macro` library:

```toml
[lib]
proc-macro = true
```

## Status

This README is broadly aligned with the current crate implementation in the workspace. It reflects the exported proc-macros in `src/lib.rs` and the validation patterns used in the repo tests under `CLR/src/tests/test_macro`.

## Main capabilities

- Generate enums/structs from INI files
- Generate structs/enums/lookups from CSV files
- Generate typed Rust structs from JSON files
- Generate state machine helpers from Mermaid/state diagrams
- Generate packet-layout structs and bit-vector helpers
- Generate function scaffolding from sequence diagrams
- Generate markdown-driven struct definitions and flow logic helpers

## Macro inventory

### INI macros

- `#[ini_enum("path/to/file.ini")]`
- `#[ini_struct("path/to/file.ini")]`
- `#[ini_enum_str("path/to/file.ini", "SomeTag")]`

### CSV macros

- `#[csv_struct("path/to/file.csv")]`
- `#[csv_struct2("path/to/file.csv", "SomeTag")]`
- `#[csv2enum_variants("path/to/file.csv", TagIdent)]`
- `csv2lookup!("path/to/file.csv", KeyCol, ValueCol, EnumName)`
- `#[csv2enum_lookup("path/to/file.csv", EnumIdent, "ValueCol")]`
- `csv2hash!("path/to/file.csv", KeyCol, ValueCol)`

### JSON macros

- `json_struct!("path/to/file.json", TypeName)`
- `json_struct2!("path/to/file.json", "RootName", TypeName)`

### Mermaid flow/state macros

- `#[flow_enum("path/to/flow.mmd")]`
- `#[state_struct("path/to/state.mmd")]`
- `#[state_struct_trait("path/to/state.mmd")]`
- `state_type_mapping!("path/to/state.mmd", TypeTag)`

### Mermaid packet/sequence macros

- `#[packet_struct("path/to/packet.mmd")]`
- `#[packet_bit_vec("path/to/packet.mmd")]`
- `#[sequence2function("path/to/sequence.mmd")]`

### Markdown / flow helper macros

- `#[md2struct("path/to/file.md", "RootType")]`
- `flow2logic!("path/to/flow.mmd", FlowType, EventEnum, StateEnum)`

## Notes on arguments

Most macros parse arguments as:

- `"path"` (string literal)
- optional extra identifiers (`Tag`, `EnumName`, `TypeName`, etc.)
- some forms accept a second string-literal tag

The parser for macro arguments is implemented in `src/init_args.rs`.

## Quick start

A typical workflow is to keep source artifacts beside the Rust module that consumes them, then invoke the macro at compile time.

```rust
use rust_macro_internal::*;

json_struct!("test_data/json/test0.json", JsonStruct);

#[csv_struct("test_data/csv/test2.csv")]
pub struct PersonRow {}

const LOOKUP: std::sync::LazyLock<std::collections::HashMap<i64, i64>> =
    std::sync::LazyLock::new(|| csv2hash!("test_data/csv/test2.csv", id, age));

fn main() {
    let user = JsonStruct::new(
        "Alice".to_string(),
        30,
        false,
        vec![95, 88, 76],
    );

    assert_eq!(user.name, "Alice");
    assert_eq!(LOOKUP[&2], 28);
}
```

This pattern is common in the CLR workspace: generated structs are created from data files and then consumed like regular Rust types.

## Usage examples

### JSON generated struct

This pattern is used directly by the repo tests in `CLR/src/tests/test_macro/test_json_macro.rs`:

```rust
use rust_macro_internal::*;

json_struct!("test_data/json/test0.json", JsonStruct);

fn example() {
    let value = JsonStruct::new(
        "Alice".to_string(),
        30,
        false,
        vec![95, 88, 76],
    );

    assert_eq!(value.name, "Alice");
}
```

### CSV generated lookup and typed rows

```rust
use rust_macro_internal::*;

#[csv_struct("test_data/csv/test2.csv")]
pub struct PersonRow {}

let lookup = csv2hash!("test_data/csv/test2.csv", id, age);
assert_eq!(lookup[&2], 28);
```

### INI generated struct

```rust
use rust_macro_internal::*;

#[ini_struct("test_data/ini/struct0.ini")]
pub struct Settings;

fn example() {
    let settings = Settings::new();
    let _ = settings;
}
```

### State machine generation

```rust
use rust_macro_internal::{state_struct, state_struct_trait, state_type_mapping};

#[state_struct("test_data/meriad/state/state0.mermaid")]
pub struct StateMachine0 {}

#[state_struct_trait("test_data/meriad/state/state0.mermaid")]
pub struct StateMachine1 {}

fn example() {
    let mut machine = StateMachine0 {
        current_state: StateMachine0StateEnum::Start,
    };

    machine.transit(&StateMachine0TriggerEvents::StartToStill);
}
```

### Packet layout generation

```rust
use rust_macro_internal::*;

#[packet_struct("src/r5asm/elf_section/program_header/program_header.mermaid")]
pub struct ProgramHeader {}
```

## Real project examples

The CLR workspace uses these macros directly in the ELF/ISA code generation pipeline.

### CSV-backed instruction metadata

This is used in `riscv_asm_lib` to load instruction metadata from a CSV file and compute changed registers for a parsed instruction:

```rust
use rust_macro_internal::csv_struct;

#[csv_struct("src/r5asm/meta/instruction_register.csv")]
pub struct InstructionRegisterMeta {}

fn example() {
    let rows = InstructionRegisterMeta::to_typed_vec();
    assert!(!rows.is_empty());
}
```

This matches the actual code in [CLR/riscv_asm_lib/src/r5asm/meta/instruction_register_meta.rs](C:/MyCode/CodeCenter/F#/RiscVGit/CLR/riscv_asm_lib/src/r5asm/meta/instruction_register_meta.rs).

### Packet layout generation for ELF program headers

This pattern is used to define binary-safe packet/struct layouts from Mermaid diagrams:

```rust
use rust_macro_internal::*;

#[packet_struct("src/r5asm/elf_section/program_header/program_header.mermaid")]
#[repr(C)]
pub struct ProgramHeader {}

fn example() {
    let header = ProgramHeader {
        segment_type: 1,
        segment_flags: 5,
        file_offset: 0,
        virtual_address: 0x1000,
        physical_address: 0x1000,
        file_size: 64,
        memory_size: 64,
        alignment: 0x1000,
    };

    let _ = header;
}
```

This mirrors the pattern in [CLR/riscv_asm_lib/src/r5asm/elf_section/program_header/program_header.rs](C:/MyCode/CodeCenter/F#/RiscVGit/CLR/riscv_asm_lib/src/r5asm/elf_section/program_header/program_header.rs).

## Build and check

From `CLR/`:

```bash
cargo check -p rust_macro_internal
cargo test -p rust_macro_internal
```

## Internal crate notice

`rust_macro_internal` is intended for workspace/internal use, and macro signatures may evolve with parser/codegen changes.
