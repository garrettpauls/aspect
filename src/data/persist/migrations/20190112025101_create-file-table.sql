create table File
( id     integer not null primary key autoincrement
, name   text    not null unique
, rating integer     null
);