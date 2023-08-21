import sqlite3
import unittest
import time
import os

EXT_PATH = "./dist/debug/xml0"


def connect(ext):
    db = sqlite3.connect(":memory:")

    db.execute("create table base_functions as select name from pragma_function_list")
    db.execute("create table base_modules as select name from pragma_module_list")

    db.enable_load_extension(True)
    db.load_extension(ext)

    db.execute(
        "create temp table loaded_functions as select name from pragma_function_list where name not in (select name from base_functions) group by 1 order by name"
    )
    db.execute(
        "create temp table loaded_modules as select name from pragma_module_list where name not in (select name from base_modules) order by name"
    )

    db.row_factory = sqlite3.Row
    return db


db = connect(EXT_PATH)


def explain_query_plan(sql):
    return db.execute("explain query plan " + sql).fetchone()["detail"]


def execute_all(sql, args=None):
    if args is None:
        args = []
    results = db.execute(sql, args).fetchall()
    return list(map(lambda x: dict(x), results))


FUNCTIONS = [
    "xml_attr_get",
    "xml_attribute_get",
    "xml_debug",
    "xml_extract",
    "xml_version",
]

MODULES = [
    "xml_each",
]


class TestXml(unittest.TestCase):
    def test_funcs(self):
        funcs = list(
            map(
                lambda a: a[0],
                db.execute("select name from loaded_functions").fetchall(),
            )
        )
        self.assertEqual(funcs, FUNCTIONS)

    def test_modules(self):
        modules = list(
            map(
                lambda a: a[0], db.execute("select name from loaded_modules").fetchall()
            )
        )
        self.assertEqual(modules, MODULES)

    def test_xml_version(self):
        version = "v0.1.0"
        self.assertEqual(db.execute("select xml_version()").fetchone()[0], version)

    def test_xml_debug(self):
        debug = db.execute("select xml_debug()").fetchone()[0]
        self.assertEqual(len(debug.splitlines()), 2)

    def test_xml_attribute_get(self):
        self.skipTest("asdf")

    def test_xml_attr_get(self):
        self.skipTest("asdf")

    def test_xml_valid(self):
        self.skipTest("asdf")
        xml_valid = lambda pattern: db.execute(
            "select xml_valid(?)", [pattern]
        ).fetchone()[0]
        self.assertEqual(xml_valid("[0-9]{3}-[0-9]{3}-[0-9]{4}"), 1)
        self.assertEqual(xml_valid("no("), 0)

    def test_xml_extract(self):
        self.skipTest("asdf")
        xml_extract = lambda document, xpath: db.execute(
            "select xml_extract(?, ?)", [document, xpath]
        ).fetchone()[0]
        self.assertEqual(xml_extract(""), 1)
        self.assertEqual(xml_valid("no("), 0)

    def test_xml_each(self):
        xml_each = lambda document, xpath: execute_all(
            "select rowid, * from xml_each(?, ?)", [document, xpath]
        )
        self.assertEqual(
            xml_each(
                """
        <items>
          <item><element>yo!</element></item>
          <item></item>
          <item>hello</item>
        </items>
      """,
                "//items/item",
            ),
            [
                {
                    "rowid": 0,
                    "xml": "<item><element>yo!</element></item>",
                    "text": "yo!",
                },
                {"rowid": 1, "xml": "<item/>", "text": ""},
                {"rowid": 2, "xml": "<item>hello</item>", "text": "hello"},
            ],
        )


class TestCoverage(unittest.TestCase):
    def test_coverage(self):
        test_methods = [method for method in dir(TestXml) if method.startswith("test_")]
        funcs_with_tests = set([x.replace("test_", "") for x in test_methods])

        for func in FUNCTIONS:
            self.assertTrue(
                func in funcs_with_tests,
                f"{func} does not have corresponding test in {funcs_with_tests}",
            )

        for module in MODULES:
            self.assertTrue(
                module in funcs_with_tests,
                f"{module} does not have corresponding test in {funcs_with_tests}",
            )


if __name__ == "__main__":
    unittest.main()
