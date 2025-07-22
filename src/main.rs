use schema_generator::{Field, Schema, Table};

fn main() {
    
    let user_table_name = "User";
    let user_table_fields = vec![
        Field::id("user_id"),
        Field::unique_string_variable_length("username")
    ];
    let users_table = Table::new(
        user_table_name,
        user_table_fields
    );

    let schema = Schema::new(vec![
        users_table
    ]);

    println!("{}", schema.sqlitize())
}
