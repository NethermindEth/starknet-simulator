# Starknet Transaction Simulator API

## Getting Started

### Prerequisites

- Install [Rust](https://www.rust-lang.org/tools/install)
- Setup Rust:

```bash
rustup override set stable && rustup update
```

Ensure rust was installed correctly by running the following from the root project directory:

```bash
cargo test
```

### Running the project

```bash
cargo build
cargo run
```

## API Usage Guide

The API runs on `http://localhost:8080` and provides the following endpoints:

### 1. Compile Cairo Code

- **Endpoint:** `/compile`
- **Method:** POST
- **Content-Type:** application/json
- **Request Body:**

```json
{
  "code": "YOUR_CAIRO_CODE",
  "file_name": "YOUR_CAIRO_FILE_NAME"
}
```

- **Response:** JSON object with compilation results

```json
{
  "cairo_sierra": {
    "contract": ...,
    "program": {...},
    "sierra_cairo_info_mapping": {...}
  },
  "casm_sierra": {
    "casm_sierra_mapping_instruction": {
        "casm_instructions": [...],
        "casm_sierra_mapping": {...}
    },
    "casm": ...
  }
}
```

### 2. Compile Cairo Contract

- **Endpoint:** `/compile_contract`
- **Method:** POST
- **Content-Type:** application/json
- **Request Body:**

```json
{
  "code": "YOUR_CAIRO_CODE",
  "file_name": "YOUR_CAIRO_FILE_NAME"
}
```

- **Response:** JSON object with compilation results

```json
{
  "cairo_sierra": {
    "contract": ...,
    "sierra_contract_class": {...},
    "sierra_cairo_info_mapping": {...}
  },
  "casm_sierra": {
    "casm_sierra_mapping_instruction": {
        "casm_instructions": [...],
        "casm_sierra_mapping": {...}
    },
    "casm_contract_class": {...}
  }
}
```

### 3. Trace Error

This returns the execution trace of a given transaction.
NOTE: Current implementation only works for failing transactions.

- **Endpoint:** `/trace_error`
- **Method:** POST
- **Content-Type:** application/json
- **Request Body:**

```json
{
  "args": ["arg1", "arg2"],
  "casm_contract_class": "Serialized CasmContractClass JSON string",
  "entrypoint_offset": 0
}
```

- **Response:** JSON object with execution trace

```json
{
    "retdata": ...,
    "trace": [{
        "pc": ...,
        "ap": ...,
        "fp": ...
  },
  ...],
}
```
