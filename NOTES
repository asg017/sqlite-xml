- [ ] subtype 'X' in `xml_extract` (if node??)
- [ ] `xml_valid(document)`
- [ ] `xml_attribute_has(document, xpath, attribute)`
- [ ] `xml_valid(document)`
- [ ] `xml_valid(document)`
- [ ] `xml_valid(document)`
- [ ] `xml_element(node, attributes, child1, ...)`

- [ ] xml schema stuff?

```sql
create table xml_reader(
  each='//item',
  './/guid' as id,
  './/name' as name,
  './/age' as age,
  './/grades[0].score' as last_score
);
```

```sql

--- how handle namespaces

select xml_extract(
  document,
  '//mediawiki:page',
  xml_namespaces(
    "mediawiki", "http://www.mediawiki.org/xml/export-0.10/"
  )
);

insert into xml_namespaces
  select "mediawiki", "http://www.mediawiki.org/xml/export-0.10/";

```
