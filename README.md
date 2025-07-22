# nosql sql rust orm

## I have a unique situation

- I am used to using DynamoDB
- I want to self host the DB
- I want a large number (100+) concurrent transactions (DynamoDB supports only 30)
- I could use Postgres, but I dont want to hand create a schema and I dont want to worry too much about schema upgrades. and I dont want to worry too much about all the complexities that postgres has, like I do not  need to join tables at all
- I also want to generate simple row insertion logic from a PORS (Plain Old Rust Struct) like `item.insert_self_into_table()` instead of coming up with the syntax myself

## Solution: Custom ORM
