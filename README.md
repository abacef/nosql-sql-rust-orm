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

The example is we want to store users, the time they were created and their interests. We can create a users table:
```rust
use schema_generator::{Field, Table};

let user_table_name = "User";
let user_id_field = "user_id";
let user_table_fields = vec![
    Field::id(user_id_field),
    Field::unique_string_variable_length("username"),
    Field::date_time("date_created")
];
let users_table = Table::new(
    user_table_name,
    user_table_fields
);
```

Instead of storing an array in a row of the users table, lets create a join table for their interests
```rust
use schema_generator::{Field, Table};

let interest_table_name = "Interest";
let interest_id_field = "interest_id";
let interest_table_fields = vec![
    Field::id(interest_id_field),
    Field::unique_string_variable_length("interest_name")
];
let interests_table = Table::new(interest_table_name, interest_table_fields);

let user_interests_join_table = Table::new_join_table(user_table_name, user_id_field, interest_table_name, interest_id_field);
```

With these two tables written in a proc macro called `generate_sql_schema`, we can call the proc macro from another crate to generate the code

```rust
generate_sql_schema!();
```

Now we can initialize the database and add a user and their interests. Note how we do not need to write sql statements for many of the desired logic. Also note how the sql schema statements have been generated in a string where we can just run them to create the tables.
```rust
// initialize the database with the sql schema
db_client.execute(DB_SCHEMA_SQL).unwrap();

// Insert one user
let now_utc = Utc::now();
let user = User::new(None, "Mike".to_string(), now_utc);
user.insert_self_into_table(&db_client);

let user_id = db_client.execute("SELECT user_id from User where username = Mike");

InterestForUser::new(user_id, "Hockey").insert_self_into_table(&db_client);
InterestForUser::new(user_id, "Coding").insert_self_into_table(&db_client);

let interests: Vec<String> = InterestForUser::get_interests(user_id, &db_client);
println!("{interests}");

let retrieved_user = User::get_from_table(user_id, &db_client);
User::delete_from_table(user_id, &db_client);
```

- I have implemented the logic to insert an item into the database with the method `insert_self_into_table` on the generated struct that shares the same name with the table's name.
- I have not currently implemented the `get_<join_table_item>s` to get the list of all items that exist for a key in a join table
- I have not yet implemented the `get_from_table` function to get a item by its primary key
- I have not yet implemented the `delete_from_table` to delete an item by its primary key
