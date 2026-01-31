# Error Model

This document describes the standard error types and reporting utilities used
by NEPLG2. The goal is a consistent, Result-first error flow that works in both
WASM and WASI targets without relying on GC.

## Core Types

`std/error` defines:

- `ErrorKind`: classification (Failure, IoError, ParseError, etc.).
- `Span`: `(file_id, start, end)` byte range.
- `Error`: `{ kind, msg, span, source }` with optional source chaining.

Errors are values carried through `Result<T, Error>`. A source chain allows
context to be attached while preserving the original cause.

## Source Locations

`callsite_span` is an intrinsic that returns a `Span` for the current call site.
Helpers like `fail` and `context` attach this span automatically.

## Reporting

`std/diag` provides:

- `diag_to_string(e) -> str`: build a human-readable report string.
- `diag_print(e)` / `diag_println(e)` (WASI only): print via stdio.

On the WASM target, diagnostics are returned as strings (no I/O). On WASI,
diagnostics can be printed to stdout/stderr via `std/stdio`.

## Ownership / No GC

All error values are explicit. There is no hidden global error state. Error
objects and their source chains are heap-allocated and managed through the
existing allocator (`std/mem`).
