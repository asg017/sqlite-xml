use libxml::readonly::RoNode;
use sqlite_loadable::prelude::*;
use sqlite_loadable::{api, Error, Result};

use libxml::parser::Parser;
use libxml::tree::{Document, NodeType};
use libxml::xpath::Context;

use crate::each::Item;

pub fn maybe_register_parameter_namespaces(
    ctx: &Context,
    value: Option<&*mut sqlite3_value>,
) -> Result<()> {
    if let Some(value) = value {
        let contents = api::value_text(value)?;
        let object = serde_json::from_str(contents)
            .map_err(|_| Error::new_message("object not valid JSON"))?;
        match object {
            serde_json::Value::Object(data) => {
                for (key, value) in data {
                    if let Some(value) = value.as_str() {
                        ctx.register_namespace(key.as_str(), value)
                            .map_err(|_| Error::new_message("asdf"))?;
                    } else {
                        return Err(Error::new_message("asdf"));
                    }
                }
            }
            _ => return Err(Error::new_message("asdf")),
        }
    }
    Ok(())
}

fn result_xml(context: *mut sqlite3_context, doc: &Document, node: &RoNode) -> Result<()> {
    api::result_text(context, doc.ronode_to_string(node).as_str())?;
    api::result_subtype(context, b'X');
    Ok(())
}
pub fn xml_extract(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let arg_doc = values.get(0).ok_or_else(|| Error::new_message("asdf"))?;
    let xpath = api::value_text(values.get(1).ok_or_else(|| Error::new_message("asdf"))?)?;

    let p = unsafe { api::value_pointer::<Item>(arg_doc, b"xml_node\0") };
    if let Some(item) = p {
        let item = unsafe { &*item };
        let result = (item).context.node_evaluate_readonly(xpath, *(item).node);
        let result = match result {
            Ok(result) => result,
            Err(_) => {
                return Err(Error::new_message(
                    format!("unknown error evaluation xpath: {}", xpath).as_str(),
                ));
            }
        };
        let matching = result.get_readonly_nodes_as_vec();
        match matching.get(0) {
            Some(node) => match node.get_type() {
                Some(NodeType::TextNode) => {
                    result_xml(context, &item.document, node)?;
                }
                _ => {
                    let value = item.document.ronode_to_string(node);
                    api::result_text(context, value)?;
                }
            },
            None => {
                api::result_null(context);
            }
        }
        return Ok(());
    }
    //let item = unsafe { Box::from_raw(p as *mut Item) };
    //let result = .unwrap();

    let parser = Parser::default();
    let doc = parser.parse_string(api::value_text(arg_doc)?).unwrap();
    let ctx = Context::new(&doc).unwrap();
    //maybe_register_parameter_namespaces(&ctx, values.get(2));
    //ctx.register_namespace("mediawiki", "http://www.mediawiki.org/xml/export-0.10/").unwrap();
    //ctx.register_namespace("media", "http://search.yahoo.com/mrss/").unwrap();

    let result = ctx
        .evaluate(api::value_text(
            values.get(1).ok_or_else(|| Error::new_message("asdf"))?,
        )?)
        .map_err(|_| Error::new_message("XPath error"))?;

    let matching = result.get_readonly_nodes_as_vec();
    match matching.get(0) {
        Some(node) => {
            println!("{:?}", node.get_type());
            api::result_text(context, doc.ronode_to_string(node).as_str())?;
        }
        None => {
            api::result_null(context);
        }
    }

    Ok(())
}

pub fn libxml_attribute_get(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
) -> Result<()> {
    let doc = api::value_text(values.get(0).ok_or_else(|| Error::new_message("asdf"))?)?;
    let xpath = api::value_text(values.get(1).ok_or_else(|| Error::new_message("asdf"))?)?;
    let attribute = api::value_text(values.get(2).ok_or_else(|| Error::new_message("asdf"))?)?;

    let parser = Parser::default();
    let doc = parser.parse_string(doc).unwrap();
    let ctx = Context::new(&doc).unwrap();
    maybe_register_parameter_namespaces(&ctx, values.get(3))?;
    let result = ctx
        .evaluate(xpath)
        .map_err(|_| Error::new_message("XPath error"))?;

    let matching = result.get_readonly_nodes_as_vec();
    match matching.get(0) {
        Some(node) => match node.get_attribute(attribute) {
            Some(value) => {
                api::result_text(context, value)?;
            }
            None => {
                api::result_null(context);
            }
        },
        None => {
            api::result_null(context);
        }
    }

    Ok(())
}
