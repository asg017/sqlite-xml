mod each;
mod scalar;

pub use crate::{
    each::XmlEachTable,
    scalar::{libxml_attribute_get, xml_extract},
};

use sqlite_loadable::prelude::*;
use sqlite_loadable::{
    api, define_scalar_function, table::define_table_function_with_find, FunctionFlags, Result,
};

use libxml::bindings::{initGenericErrorDefaultFunc, xmlGenericErrorFunc};
use libxml::tree::{Document, Node};

pub fn xml_version(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    api::result_text(context, format!("v{}", env!("CARGO_PKG_VERSION")))?;
    Ok(())
}

pub fn xml_debug(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    api::result_text(
        context,
        format!(
            "Version: v{}
Source: {}
",
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH")
        ),
    )?;
    Ok(())
}

unsafe extern "C" fn err(_ctx: *mut ::std::os::raw::c_void, _msg: *const ::std::os::raw::c_char) {}

fn debug() {
    let doc_result = Document::new();
    assert!(doc_result.is_ok());
    let mut doc = doc_result.unwrap();

    // This tests for functionality (return self if there is no root element) that is removed.
    let doc_node = doc.get_root_element();
    assert_eq!(doc_node, None, "empty document has no root element");

    let hello_element_result = Node::new("hello", None, &doc);
    assert!(hello_element_result.is_ok());
    let mut hello_element = hello_element_result.unwrap();
    hello_element.set_attribute("name", "alex").unwrap();

    assert!(hello_element.set_content("world!").is_ok());

    doc.set_root_element(&hello_element);

    assert!(hello_element.set_content("world!").is_ok());

    let added = hello_element.new_child(None, "child");
    assert!(added.is_ok());
    let mut new_child = added.unwrap();

    assert!(new_child.set_content("set content").is_ok());

    assert_eq!(new_child.get_content(), "set content");
    assert_eq!(hello_element.get_content(), "world!set content");

    let node_string = doc.node_to_string(&hello_element);
}
#[sqlite_entrypoint]
pub fn sqlite3_xml_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    let pointer = err as *const ();
    unsafe {
        let mut errx = std::mem::transmute::<*const (), xmlGenericErrorFunc>(pointer);
        initGenericErrorDefaultFunc(&mut (errx));
    }
    define_scalar_function(db, "xml_version", 0, xml_version, flags)?;
    define_scalar_function(db, "xml_debug", 0, xml_debug, flags)?;

    define_scalar_function(db, "xml_extract", 2, xml_extract, flags)?;
    define_scalar_function(db, "xml_extract", 3, xml_extract, flags)?;

    define_scalar_function(db, "xml_attribute_get", 3, libxml_attribute_get, flags)?;
    define_scalar_function(db, "xml_attr_get", 3, libxml_attribute_get, flags)?;

    define_table_function_with_find::<XmlEachTable>(db, "xml_each", None)?;

    Ok(())
}
