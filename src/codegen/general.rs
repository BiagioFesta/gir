use std::fmt::Display;
use std::io::{Result, Write};

use analysis::general::StatusedTypeId;
use analysis::imports::Imports;
use config::Config;
use git::repo_hash;
use gir_version::VERSION;
use nameutil::crate_name;
use version::Version;
use writer::primitives::tabs;

pub fn start_comments(w: &mut Write, conf: &Config) -> Result<()>{
    try!(writeln!(w, "// This file was generated by gir ({}) from gir-files ({})",
            VERSION, repo_hash(&conf.girs_dir).unwrap_or_else(|_| "???".into())));
    try!(writeln!(w, "// DO NOT EDIT"));

    Ok(())
}

pub fn uses(w: &mut Write, imports: &Imports, library_name: &str, min_cfg_version: Version)
        -> Result<()>{
    try!(writeln!(w, ""));
    for (name, version) in imports.iter() {
        try!(version_condition(w, library_name, min_cfg_version, version.clone(), false, 0));
        try!(writeln!(w, "use {};", name));
    }

    Ok(())
}

pub fn define_object_type(w: &mut Write, type_name: &str, glib_name: &str) -> Result<()>{
    try!(writeln!(w, ""));
    try!(writeln!(w, "pub type {} = Object<ffi::{}>;", type_name, glib_name));

    Ok(())
}

pub fn define_boxed_type(w: &mut Write, type_name: &str, glib_name: &str,
                                copy_fn: &str, free_fn: &str) -> Result<()>{
    try!(writeln!(w, ""));
    try!(writeln!(w, "glib_wrapper! {{"));
    try!(writeln!(w, "\tpub struct {}(Boxed<ffi::{}>);", type_name, glib_name));
    try!(writeln!(w, ""));
    try!(writeln!(w, "\tmatch fn {{"));
    try!(writeln!(w, "\t\tcopy => |ptr| ffi::{}(ptr as *mut ffi::{}),",
                  copy_fn, glib_name));
    try!(writeln!(w, "\t\tfree => |ptr| ffi::{}(ptr),", free_fn));
    try!(writeln!(w, "\t}}"));
    try!(writeln!(w, "}}"));

    Ok(())
}

pub fn impl_parents(w: &mut Write, type_name: &str, parents: &[StatusedTypeId]) -> Result<()>{
    try!(writeln!(w, ""));
    for stid in parents {
        //TODO: don't generate for parents without traits
        if !stid.status.ignored() {
            try!(writeln!(w, "unsafe impl Upcast<{}> for {} {{ }}", stid.name, type_name));
        }
    }

    Ok(())
}

pub fn impl_interfaces(w: &mut Write, type_name: &str, implements: &[StatusedTypeId]) -> Result<()>{
    for stid in implements {
        if !stid.status.ignored() {
            try!(writeln!(w, "unsafe impl Upcast<{}> for {} {{ }}", stid.name, type_name));
        }
    }

    Ok(())
}

pub fn impl_static_type(w: &mut Write, type_name: &str, glib_func_name: &str) -> Result<()>{
    try!(writeln!(w, ""));
    try!(writeln!(w, "impl types::StaticType for {} {{", type_name));
    try!(writeln!(w, "\t#[inline]"));
    try!(writeln!(w, "\tfn static_type() -> types::Type {{"));
    try!(writeln!(w, "\t\tunsafe {{ from_glib(ffi::{}()) }}", glib_func_name));
    try!(writeln!(w, "\t}}"));
    try!(writeln!(w, "}}"));

    Ok(())
}

pub fn version_condition(w: &mut Write, library_name: &str, min_cfg_version: Version,
        version: Option<Version>, commented: bool, indent: usize) -> Result<()> {
    let s = version_condition_string(library_name, min_cfg_version, version, commented, indent);
    if let Some(s) = s {
        try!(writeln!(w, "{}", s));
    }
    Ok(())
}

pub fn version_condition_string(library_name: &str, min_cfg_version: Version,
        version: Option<Version>, commented: bool, indent: usize) -> Option<String> {
    match version {
        Some(v) if v >= min_cfg_version => {
            let comment = if commented { "//" } else { "" };
            Some(format!("{}{}#[cfg({})]", tabs(indent), comment,
                v.to_cfg(&crate_name(library_name))))
        }
        _ => None
    }
}

pub fn write_vec<T: Display>(w: &mut Write, v: &[T]) -> Result<()> {
    for s in v {
        try!(writeln!(w, "{}", s));
    }
    Ok(())
}
