mod each;
mod meta;
mod scalar;

pub use crate::{
    each::XmlEachTable,
    meta::{xml_debug, xml_version},
    scalar::{libxml_attribute_get, xml_extract},
};

use sqlite_loadable::prelude::*;
use sqlite_loadable::{
    define_scalar_function, define_table_function, errors::Result, FunctionFlags,
};

use libxml::bindings::{initGenericErrorDefaultFunc, xmlGenericErrorFunc};

unsafe extern "C" fn err(_ctx: *mut ::std::os::raw::c_void, _msg: *const ::std::os::raw::c_char) {}

#[sqlite_entrypoint]
pub fn sqlite3_xml_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    let pointer = err as *const ();
    let mut errx = unsafe { std::mem::transmute::<*const (), xmlGenericErrorFunc>(pointer) };
    unsafe {
        initGenericErrorDefaultFunc(&mut (errx));
    }
    define_scalar_function(db, "xml_version", 0, xml_version, flags)?;
    define_scalar_function(db, "xml_debug", 0, xml_debug, flags)?;

    define_scalar_function(db, "xml_extract", 2, xml_extract, flags)?;
    define_scalar_function(db, "xml_extract", 3, xml_extract, flags)?;
    define_scalar_function(db, "xml_attribute_get", 3, libxml_attribute_get, flags)?;

    define_table_function::<XmlEachTable>(db, "xml_each", None)?;

    Ok(())
}
