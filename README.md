# sqlite-xml

A work-in-progress SQLite extension for querying XML! Not meant to be widely shared.

Once it's ready, you'll be able to do things like:

```sql
select xml_extract(readfile('student.xml'), '//student/name/text()'); -- 'Alex Garcia'

select
  xml_extract(node, './/text()') as text
from xml_each(
  '
    <items>
      <item>Alex</item>
      <item>Brian</item>
      <item>Craig</item>
    </items>
  ',
  '//item'
);

/*
┌───────┐
│ text  │
├───────┤
│ Alex  │
│ Brian │
│ Craig │
└───────┘
*/
```
