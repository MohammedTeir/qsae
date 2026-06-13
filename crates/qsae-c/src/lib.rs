use libc::{c_char, c_int, size_t};
use qsae_core::{Compressor, CompressorConfig, Decompressor, QuorumParams};
use std::ffi::{CStr, CString};
use std::slice;

/// QSAE C bindings.
/// Provides a C-compatible API for embedding QSAE in C/C++ applications.

/// Opaque handle for compressor configuration.
pub struct QsaeConfig {
    config: CompressorConfig,
}

/// Opaque handle for compressor.
pub struct QsaeCompressor {
    compressor: Compressor,
}

/// Opaque handle for decompressor.
pub struct QsaeDecompressor {
    decompressor: Decompressor,
}

/// Result structure for C API.
#[repr(C)]
pub struct QsaeResult {
    pub success: c_int,
    pub data: *mut u8,
    pub len: size_t,
    pub error: *mut c_char,
}

/// Compression statistics.
#[repr(C)]
pub struct QsaeStats {
    pub original_size: size_t,
    pub compressed_size: size_t,
    pub ratio: f64,
    pub block_count: size_t,
    pub duration_ms: u64,
}

/// Create a default configuration.
#[no_mangle]
pub extern "C" fn qsae_config_default() -> *mut QsaeConfig {
    let config = QsaeConfig {
        config: CompressorConfig::default(),
    };
    Box::into_raw(Box::new(config))
}

/// Create a configuration with custom parameters.
#[no_mangle]
pub extern "C" fn qsae_config_new(lambda: f64, delta: f64, block_size: size_t) -> *mut QsaeConfig {
    let config = QsaeConfig {
        config: CompressorConfig::builder()
            .quorum(QuorumParams::new().with_lambda(lambda).with_delta(delta))
            .block_size_hint(block_size)
            .build(),
    };
    Box::into_raw(Box::new(config))
}

/// Free a configuration.
#[no_mangle]
pub extern "C" fn qsae_config_free(config: *mut QsaeConfig) {
    if !config.is_null() {
        unsafe { drop(Box::from_raw(config)) };
    }
}

/// Create a compressor.
#[no_mangle]
pub extern "C" fn qsae_compressor_new(config: *mut QsaeConfig) -> *mut QsaeCompressor {
    let config = unsafe {
        if config.is_null() {
            return std::ptr::null_mut();
        }
        &*config
    };

    let compressor = QsaeCompressor {
        compressor: Compressor::new(config.config.clone()),
    };
    Box::into_raw(Box::new(compressor))
}

/// Free a compressor.
#[no_mangle]
pub extern "C" fn qsae_compressor_free(compressor: *mut QsaeCompressor) {
    if !compressor.is_null() {
        unsafe { drop(Box::from_raw(compressor)) };
    }
}

/// Compress data.
#[no_mangle]
pub extern "C" fn qsae_compress(
    compressor: *mut QsaeCompressor,
    input: *const u8,
    input_len: size_t,
) -> QsaeResult {
    let compressor = unsafe {
        if compressor.is_null() {
            return make_error("Null compressor handle");
        }
        &mut *compressor
    };

    let input = unsafe { slice::from_raw_parts(input, input_len) };

    match compressor.compressor.compress(input) {
        Ok(data) => {
            let boxed_slice = data.into_boxed_slice();
            let len = boxed_slice.len();
            let ptr = Box::into_raw(boxed_slice) as *mut u8;
            QsaeResult {
                success: 1,
                data: ptr,
                len,
                error: std::ptr::null_mut(),
            }
        }
        Err(e) => make_error(&format!("Compression failed: {}", e)),
    }
}

/// Compress a file.
#[no_mangle]
pub extern "C" fn qsae_compress_file(
    compressor: *mut QsaeCompressor,
    input_path: *const c_char,
    output_path: *const c_char,
    stats: *mut QsaeStats,
) -> c_int {
    let compressor = unsafe {
        if compressor.is_null() {
            return -1;
        }
        &mut *compressor
    };

    let input = unsafe { CStr::from_ptr(input_path).to_string_lossy() };
    let output = unsafe { CStr::from_ptr(output_path).to_string_lossy() };

    match compressor.compressor.compress_file(&input, &output) {
        Ok(s) => {
            if !stats.is_null() {
                unsafe {
                    (*stats).original_size = s.original_size;
                    (*stats).compressed_size = s.compressed_size;
                    (*stats).ratio = s.ratio;
                    (*stats).block_count = s.block_count;
                    (*stats).duration_ms = s.duration_ms;
                }
            }
            0
        }
        Err(_) => -1,
    }
}

/// Create a decompressor.
#[no_mangle]
pub extern "C" fn qsae_decompressor_new() -> *mut QsaeDecompressor {
    let decompressor = QsaeDecompressor {
        decompressor: Decompressor::new(),
    };
    Box::into_raw(Box::new(decompressor))
}

/// Free a decompressor.
#[no_mangle]
pub extern "C" fn qsae_decompressor_free(decompressor: *mut QsaeDecompressor) {
    if !decompressor.is_null() {
        unsafe { drop(Box::from_raw(decompressor)) };
    }
}

/// Decompress data.
#[no_mangle]
pub extern "C" fn qsae_decompress(
    decompressor: *mut QsaeDecompressor,
    input: *const u8,
    input_len: size_t,
) -> QsaeResult {
    let decompressor = unsafe {
        if decompressor.is_null() {
            return make_error("Null decompressor handle");
        }
        &mut *decompressor
    };

    let input = unsafe { slice::from_raw_parts(input, input_len) };

    match decompressor.decompressor.decompress(input) {
        Ok(data) => {
            let boxed_slice = data.into_boxed_slice();
            let len = boxed_slice.len();
            let ptr = Box::into_raw(boxed_slice) as *mut u8;
            QsaeResult {
                success: 1,
                data: ptr,
                len,
                error: std::ptr::null_mut(),
            }
        }
        Err(e) => make_error(&format!("Decompression failed: {}", e)),
    }
}

/// Decompress a file.
#[no_mangle]
pub extern "C" fn qsae_decompress_file(
    decompressor: *mut QsaeDecompressor,
    input_path: *const c_char,
    output_path: *const c_char,
) -> c_int {
    let decompressor = unsafe {
        if decompressor.is_null() {
            return -1;
        }
        &mut *decompressor
    };

    let input = unsafe { CStr::from_ptr(input_path).to_string_lossy() };
    let output = unsafe { CStr::from_ptr(output_path).to_string_lossy() };

    match decompressor.decompressor.decompress_file(&input, &output) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Free result data.
#[no_mangle]
pub extern "C" fn qsae_result_free(result: QsaeResult) {
    if !result.data.is_null() && result.len > 0 {
        unsafe {
            let slice = slice::from_raw_parts_mut(result.data, result.len);
            let _ = Box::from_raw(slice);
        }
    }
    if !result.error.is_null() {
        unsafe { drop(CString::from_raw(result.error)) };
    }
}

/// Get library version.
#[no_mangle]
pub extern "C" fn qsae_version() -> *mut c_char {
    let version = CString::new(qsae_core::VERSION).unwrap();
    version.into_raw()
}

/// Free version string.
#[no_mangle]
pub extern "C" fn qsae_version_free(version: *mut c_char) {
    if !version.is_null() {
        unsafe { drop(CString::from_raw(version)) };
    }
}

fn make_error(msg: &str) -> QsaeResult {
    let error = CString::new(msg).unwrap().into_raw();
    QsaeResult {
        success: 0,
        data: std::ptr::null_mut(),
        len: 0,
        error,
    }
}
