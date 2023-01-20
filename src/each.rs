use libxml::{parser::Parser, readonly::RoNode, tree::Document, xpath::Context};
use sqlite_loadable::{
    api,
    table::{ConstraintOperator, IndexInfo, VTab, VTabArguments, VTabCursor},
    BestIndexError, Result,
};
use sqlite_loadable::{prelude::*, Error};

use std::{
    marker::PhantomData,
    mem,
    os::raw::{c_int, c_void},
};

static CREATE_SQL: &str = "CREATE TABLE x(xml text, text text, document hidden, xpath hidden, namespaces hidden, node hidden)";
enum Columns {
    Contents,
    Text,
    Document,
    Xpath,
    Namespaces,
    Node,
}
fn column(index: i32) -> Option<Columns> {
    match index {
        0 => Some(Columns::Contents),
        1 => Some(Columns::Text),
        2 => Some(Columns::Document),
        3 => Some(Columns::Xpath),
        4 => Some(Columns::Namespaces),
        5 => Some(Columns::Node),
        _ => None,
    }
}

// The possible values of idxnum in xFilter/xBestIndex
#[derive(Debug)]
enum IndexNum {
    Base,
    BaseWithNamespaces,
}

use std::convert::{Into, TryFrom};

impl TryFrom<i32> for IndexNum {
    type Error = ();

    fn try_from(v: i32) -> std::result::Result<Self, Self::Error> {
        match v {
            x if x == IndexNum::Base as i32 => Ok(IndexNum::Base),
            x if x == IndexNum::BaseWithNamespaces as i32 => Ok(IndexNum::BaseWithNamespaces),
            _ => Err(()),
        }
    }
}

impl Into<i32> for IndexNum {
    fn into(self) -> i32 {
        match self {
            IndexNum::Base => IndexNum::Base as i32,
            IndexNum::BaseWithNamespaces => IndexNum::BaseWithNamespaces as i32,
        }
    }
}

#[repr(C)]
pub struct XmlEachTable {
    /// must be first
    base: sqlite3_vtab,
}

impl<'vtab> VTab<'vtab> for XmlEachTable {
    type Aux = ();
    type Cursor = XmlEachCursor<'vtab>;

    fn connect(
        _db: *mut sqlite3,
        _aux: Option<&()>,
        _args: VTabArguments,
    ) -> Result<(String, XmlEachTable)> {
        let base: sqlite3_vtab = unsafe { mem::zeroed() };
        let vtab = XmlEachTable { base };
        // TODO db.config(VTabConfig::Innocuous)?;
        Ok((CREATE_SQL.to_owned(), vtab))
    }
    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        let mut has_document = false;
        let mut has_xpath = false;
        let mut idxnum = IndexNum::Base;
        for mut constraint in info.constraints() {
            match column(constraint.column_idx()) {
                Some(Columns::Document) => {
                    if constraint.usable() && constraint.op() == Some(ConstraintOperator::EQ) {
                        constraint.set_omit(true);
                        constraint.set_argv_index(1);
                        has_document = true;
                    } else {
                        return Err(BestIndexError::Constraint);
                    }
                }
                Some(Columns::Xpath) => {
                    if constraint.usable() && constraint.op() == Some(ConstraintOperator::EQ) {
                        constraint.set_omit(true);
                        constraint.set_argv_index(2);
                        has_xpath = true;
                    } else {
                        return Err(BestIndexError::Constraint);
                    }
                }
                Some(Columns::Namespaces) => {
                    if constraint.usable() && constraint.op() == Some(ConstraintOperator::EQ) {
                        constraint.set_omit(true);
                        constraint.set_argv_index(3);
                        idxnum = IndexNum::BaseWithNamespaces;
                    } else {
                        return Err(BestIndexError::Constraint);
                    }
                }
                // No constraints on node pointer
                // TODO maybe regular error for error mesage?
                Some(Columns::Node) => {
                    return Err(BestIndexError::Constraint);
                }
                _ => (),
            }
        }
        if !has_document || !has_xpath {
            return Err(BestIndexError::Error);
        }
        info.set_estimated_cost(100000.0);
        info.set_estimated_rows(100000);
        info.set_idxnum(idxnum.into());

        Ok(())
    }

    fn open(&mut self) -> Result<XmlEachCursor<'_>> {
        Ok(XmlEachCursor::new())
    }

    fn create(
        db: *mut sqlite3,
        aux: Option<&Self::Aux>,
        args: VTabArguments,
    ) -> Result<(String, Self)> {
        Self::connect(db, aux, args)
    }
}

#[repr(C)]
pub struct XmlEachCursor<'vtab> {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    doc: Option<Document>,
    context: Option<Context>,
    nodes: Option<Vec<RoNode>>,
    rowid: i64,
    phantom: PhantomData<&'vtab XmlEachTable>,
}
impl XmlEachCursor<'_> {
    fn new<'vtab>() -> XmlEachCursor<'vtab> {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };
        XmlEachCursor {
            base,
            doc: None,
            context: None,
            nodes: None,
            rowid: 0,
            phantom: PhantomData,
        }
    }
}

impl VTabCursor for XmlEachCursor<'_> {
    fn filter(
        &mut self,
        idx_num: c_int,
        _idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
    ) -> Result<()> {
        let document = api::value_text(values.get(0).unwrap())?;
        let xpath = api::value_text(values.get(1).unwrap())?;

        let parser = Parser::default();
        let doc = parser.parse_string(document).unwrap();
        let ctx = Context::new(&doc).unwrap();

        match idx_num.try_into() {
            Ok(IndexNum::Base) => (),
            Ok(IndexNum::BaseWithNamespaces) => {
                maybe_register_parameter_namespaces(&ctx, values.get(2))?;
            }
            Err(_) => return Err(Error::new_message("what")),
        };
        //ctx.register_namespace("mediawiki", "http://www.mediawiki.org/xml/export-0.10/");
        //ctx.register_namespace("media", "http://search.yahoo.com/mrss/");

        self.doc = Some(doc);

        let result = ctx
            .evaluate(&xpath)
            .map_err(|_| Error::new_message("XPath error"))?;
        self.context = Some(ctx);

        self.nodes = Some(result.get_readonly_nodes_as_vec());

        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.rowid += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.rowid >= self.nodes.as_ref().unwrap().len().try_into().unwrap()
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        match column(i) {
            Some(Columns::Contents) => {
                let node = self
                    .nodes
                    .as_ref()
                    .unwrap()
                    .get(self.rowid as usize)
                    .unwrap();
                api::result_text(
                    context,
                    self.doc.as_ref().unwrap().ronode_to_string(node).as_str(),
                )?;
            }
            Some(Columns::Text) => {
                let node = self
                    .nodes
                    .as_ref()
                    .unwrap()
                    .get(self.rowid as usize)
                    .unwrap();
                api::result_text(context, node.get_content().as_str())?;
            }
            Some(Columns::Node) => {
                let node = self
                    .nodes
                    .as_ref()
                    .unwrap()
                    .get(self.rowid as usize)
                    .unwrap()
                    .to_owned();
                let item = Item {
                    node: Box::new(node),
                    document: Box::new(self.doc.as_ref().unwrap().to_owned()),
                    context: Box::new(self.context.as_ref().unwrap().to_owned()),
                };
                //let p = Box::into_raw(Box::new(item));
                api::result_pointer(context, b"xml_node\0", item);
            }
            _ => (),
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.rowid.try_into().unwrap())
    }
}
unsafe extern "C" fn cleanup(p: *mut c_void) {
    drop(Box::from_raw(p.cast::<*mut Item>()))
}

pub struct Item {
    pub node: Box<RoNode>,
    pub document: Box<Document>,
    pub context: Box<Context>,
}

pub fn maybe_register_parameter_namespaces(
    ctx: &Context,
    value: Option<&*mut sqlite3_value>,
) -> Result<()> {
    if let Some(value) = value {
        let contents = api::value_text(value)?;
        let object = serde_json::from_str(contents).map_err(|err| {
            Error::new_message(format!("object not valid JSON: {}", err.to_string()).as_str())
        })?;
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
