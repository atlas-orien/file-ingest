use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};

fn path_from_cstr(ptr: *const c_char, name: &str) -> Result<PathBuf, String> {
    if ptr.is_null() {
        return Err(format!("{name} is null"));
    }
    let cstr = unsafe { CStr::from_ptr(ptr) };
    let s = cstr
        .to_str()
        .map_err(|_| format!("{name} is not valid utf-8"))?;
    if s.is_empty() {
        return Err(format!("{name} is empty"));
    }
    Ok(PathBuf::from(s))
}

fn set_out_string(out: *mut *mut c_char, value: String) {
    if out.is_null() {
        return;
    }
    let c = match CString::new(value) {
        Ok(v) => v,
        Err(_) => return,
    };
    unsafe {
        *out = c.into_raw();
    }
}

fn build_output_path(input_path: &Path, output_dir: &Path) -> Result<PathBuf, String> {
    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "invalid input filename".to_string())?;
    Ok(output_dir.join(format!("{stem}.md")))
}

/// Convert a file to Markdown and write it to the output directory.
///
/// Returns 0 on success, non-zero on failure.
/// On success, `out_path` receives the output file path.
/// On error, `out_err` receives an error message.
#[unsafe(no_mangle)]
pub extern "C" fn file_ingest_md_convert(
    _file_type: *const c_char,
    input_path: *const c_char,
    output_dir: *const c_char,
    out_path: *mut *mut c_char,
    out_err: *mut *mut c_char,
) -> i32 {
    let input_path = match path_from_cstr(input_path, "input_path") {
        Ok(v) => v,
        Err(e) => {
            set_out_string(out_err, e);
            return 1;
        }
    };
    let output_dir = match path_from_cstr(output_dir, "output_dir") {
        Ok(v) => v,
        Err(e) => {
            set_out_string(out_err, e);
            return 1;
        }
    };

    if let Err(e) = fs::create_dir_all(&output_dir) {
        set_out_string(out_err, format!("create output dir failed: {e}"));
        return 2;
    }

    let output_path = match build_output_path(&input_path, &output_dir) {
        Ok(v) => v,
        Err(e) => {
            set_out_string(out_err, e);
            return 3;
        }
    };

    let markdown = match file_ingest::to_markdown(&input_path) {
        Ok(v) => v,
        Err(e) => {
            set_out_string(out_err, format!("convert failed: {e}"));
            return 4;
        }
    };

    if let Err(e) = fs::write(&output_path, markdown) {
        set_out_string(out_err, format!("write output failed: {e}"));
        return 5;
    }

    set_out_string(out_path, output_path.display().to_string());
    0
}

/// Free strings returned by this library.
#[unsafe(no_mangle)]
pub extern "C" fn file_ingest_md_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}
