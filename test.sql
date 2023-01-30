.load target/debug/libxml0
.header on
.mode box

select xml_extract('<a> <t>c</e>', '//t');

.exit


select 
  xml_extract(node, './/text()') as text,
  xml_extract(node, './/sub') as sub,
  xml
from xml_each(
  '<all>
    <item>a</item>
    <item><sub></sub></item>
    <item>c</item>
    <item>d</item>
  </all>', 
  '//item'
);

.exit

select 
  node,
  --contents,
  xml_extract(node, './/title/text()'),
  xml_extract(node, './/link/text()'),
  xml_extract(node, './/media:rating')
from xml_each(
  readfile('simon.rss'), 
  '//item', 
  json_object(
    "media", "http://search.yahoo.com/mrss/"
  )
)
where xml_extract(node, './/media:content') is not null;
--where xml_extract(node, './/media:content') is not null;

.exit

create table documents as 
  select '<?xml version="1.0" encoding="UTF-8"?>
<root>
    <child attribute="value">some text</child>
    <child attribute="empty">more text</child>
</root>' as document;

select 
  xml_extract(document, '//child/text()'),
  xml_extract(document, '//child'),
  --xml_extract(document, '//child'),
  xml_extract(document, "//child[@attribute][2]")
from documents;

select 
  xml_extract(document, '//child/text()'),
  xml_extract(document, '//child'),
  --xml_extract(document, '//child'),
  xml_extract(document, "//child[@attribute][2]")
from documents;

select xml_extract(
  readfile('simon.rss'),
  '//title['
);

select 
  xml_extract(contents, '//title/text()') as title,
  xml_extract(contents, '//description/text()') as description
  --contents
from xml_each(readfile('simon.rss'), '//item');

select xml_extract(
  format('%s%s%s',
    '<?xml version="1.0" encoding="UTF-8"?>',
    readfile('dump.xml'),
    ''
  ),
  '//mediawiki/page/revision'
);


select xml_extract(
  '<?xml version="1.0" encoding="UTF-8"?>
  <mediawiki xmlns="http://www.mediawiki.org/xml/export-0.10/">
    <siteinfo>
      <sitename>Wikipedia</sitename>
    </siteinfo>
  </mediawiki>
',
  '//mediawiki:sitename'
);
select xml_extract(
  readfile('dump.xml'),
  '//mediawiki:siteinfo/mediawiki:sitename'
);

.timer on
--select count(*) from xml_each(readfile('dump.xml'), '//mediawiki:page/mediawiki:revision');



select 
  --contents,
  xml_extract(node, './/mediawiki:title/text()') as title,
  xml_extract(node, './/mediawiki:timestamp/text()') as timestamp
from xml_each(
  readfile('enwikinews-latest-pages-articles.xml'), 
  '//mediawiki:page',
  json_object("mediawiki", "http://www.mediawiki.org/xml/export-0.10/")
)
order by 2 desc
limit 20;

-- curl -d "&pages=Main_Page&action=submit&limit=1000" https://en.wikipedia.org/w/index.php?title=Special:Export -o dump.xml