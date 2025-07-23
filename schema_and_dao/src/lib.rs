use proc_macro::TokenStream;

use quote::{format_ident, quote};
use schema_generator::{Field, Schema, Table};

fn build_schema() -> Schema {
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

    let interest_table_name = "Interest";
    let interest_id_field = "interest_id";
    let interest_table_fields = vec![
        Field::id(interest_id_field),
        Field::unique_string_variable_length("interest_name")
    ];
    let interests_table = Table::new(interest_table_name, interest_table_fields);

    let user_interests_join_table = Table::new_join_table(user_table_name, user_id_field, interest_table_name, interest_id_field);

    Schema::new(vec![
        users_table,
        interests_table,
        user_interests_join_table
    ])
}

#[proc_macro]
pub fn generate_sql_schema(_: TokenStream) -> TokenStream {
    let code_schema = build_schema();

    let sql_schema = code_schema.sqlitize();
    println!("{}", &sql_schema);

    let structitized_schema = code_schema.structitize();
    let db_schema_ident = format_ident!("DB_SCHEMA_SQL");

    let schema_and_dao_objects = quote! {
        pub const #db_schema_ident: &str = #sql_schema;

        #structitized_schema
    };

    TokenStream::from(schema_and_dao_objects)
}