# nosql sql rust orm

## I have a unique situation

- I am used to using DynamoDB
- I want to self host the DB
- I want a large number (100+) concurrent transactions (DynamoDB is not optimized for concurrent transactions and I have hit the limit before)
- I could use Postgres, but I dont want to:
  - Use all the complex things Postgres allows you to do like joins and stored procedures
  - Hand create a schema
  - Make the database validate constraints like string lengths, uniqueness, non null. If the database operation did not succeed, it is an immediate server error. The application should be validating this
  - worry too much about schema upgrades
- I also want to generate simple row insertion logic from a PORS (Plain Old Rust Struct) like `item.insert_self_into_table()` instead of coming up with the syntax myself

## Solution: Custom ORM

- The `schema_generator` crate provides an API for a proc macro to build a sql schema and generate Rust code based on it
- The `schema_and_dao` crate defines the proc macro to generate a sql schema example (users and interests)
- The top level crate provides example code to insert a item into a table