// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::alloc::{Layout, alloc_zeroed, dealloc};
use std::io;
use std::ptr::NonNull;

const WASM_PAGE_SIZE: usize = 64 * 1024;

fn layout(size: usize) -> io::Result<Layout> {
    Layout::from_size_align(size.max(1), WASM_PAGE_SIZE)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid allocation layout"))
}

/// Reserves a memory region for the arena allocator.
///
/// WebAssembly linear memory does not expose native-style reserve/commit VM
/// operations, so this POC allocates the full backing region up front.
pub unsafe fn virtual_reserve(size: usize) -> io::Result<NonNull<u8>> {
    let layout = layout(size)?;
    let ptr = unsafe { alloc_zeroed(layout) };
    NonNull::new(ptr).ok_or_else(|| io::Error::other("wasm allocation failed"))
}

/// Releases a memory region acquired from [`virtual_reserve`].
pub unsafe fn virtual_release(base: NonNull<u8>, size: usize) {
    if let Ok(layout) = layout(size) {
        unsafe { dealloc(base.as_ptr(), layout) };
    }
}

/// Commits a memory range.
///
/// Memory is already allocated in [`virtual_reserve`] for wasm, so committing is
/// a no-op. Bounds are still enforced by the arena's capacity checks.
pub unsafe fn virtual_commit(_base: NonNull<u8>, _size: usize) -> io::Result<()> {
    Ok(())
}
