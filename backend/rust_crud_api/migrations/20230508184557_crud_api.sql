-- Add migration script here
CREATE TABLE employee (
    id  SERIAL PRIMARY KEY,
    eid varchar(255) NOT NULL,
    ename varchar(255) NOT NULL,
    eemail varchar(255) NOT NULL,
    econtact varchar(255) NOT NULL
);