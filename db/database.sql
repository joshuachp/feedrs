CREATE TABLE IF NOT EXISTS Articles (
  id TEXT,
  source TEXT,
  title TEXT NOT NULL,
  sub_title TEXT NOT NULL,
  content TEXT NOT NULL,
  date DATETIME,
  PRIMARY KEY (id, source)
);
INSERT
OR REPLACE INTO Articles (id, source, title, sub_title, content, date)
VALUES
  (
    'id',
    'source',
    'title',
    'sub_title',
    'content',
    '2016-11-08T03:50:23-05:00'
  );
