use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Type, parse_str};

const INDENT: &str = "    ";

pub struct Schema {
    tables: Vec<Table>,
}
impl Schema {
    pub fn new(tables: Vec<Table>) -> Schema {
        Schema {
            tables
        }
    }

    pub fn sqlitize(&self) -> String {
        let tables = self
            .tables
            .iter()
            .map(Table::sqlitize)
            .collect::<Vec<String>>()
            .join("\n\n");

        tables.to_string()
    }

    pub fn structitize(&self) -> TokenStream {
        let structs = self
            .tables
            .iter()
            .map(Table::structitize)
            .collect::<Vec<TokenStream>>();
        quote! {
            use rust_decimal::Decimal;
            use chrono::{DateTime, Utc};
            use tokio_postgres::Client;
            use tokio_postgres::Row;
            use r_lombok_macros::Getter;

            #(#structs)*
        }
    }
}

struct CompositeUniqueness {
    field1: String,
    field2: String,
}

pub struct Table {
    name: String,
    fields: Vec<Field>,
    composite_uniqueness: Option<CompositeUniqueness>,
    primary_key: Option<String>,
}
impl Table {
    pub fn new(name: &str, fields: Vec<Field>) -> Table {
        Self::new_full(name.to_string(), fields, None, None)
    }
    fn new_full(name: String, fields: Vec<Field>, composite_uniqueness: Option<CompositeUniqueness>, primary_key: Option<String>) -> Table {
        Table {
            name, fields, composite_uniqueness, primary_key
        }
    }
    pub fn new_join_table(table_1: &str, table_1_id: &str, table_2: &str, table_2_id: &str) -> Table {
        Table {
            name: format!("{table_2}For{table_1}"),
            fields: vec![
                Field {
                    name: table_1_id.to_string(),
                    typ: FieldType::Int,
                    nullable: false,
                    unique: false,
                    reference: Some(FieldReference {
                        referenced_table: table_1.to_string(),
                        referenced_field_in_that_table: table_1_id.to_string(),
                    }),
                },
                Field {
                    name: table_2_id.to_string(),
                    typ: FieldType::Int,
                    nullable: false,
                    unique: false,
                    reference: Some(FieldReference {
                        referenced_table: table_2.to_string(),
                        referenced_field_in_that_table: table_2_id.to_string(),
                    }),
                },
            ],
            composite_uniqueness: Some(CompositeUniqueness {
                field1: table_1_id.to_string(),
                field2: table_2_id.to_string(),
            }),
            primary_key: None,
        }
    }

    fn sqlitize(&self) -> String {
        let uniqueness_constraint_addition = match &self.composite_uniqueness {
            Some(cu) => format!(",\n{INDENT}UNIQUE ({}, {})", cu.field1, cu.field2),
            None => String::new(),
        };

        format!(
            "CREATE TABLE {} (\n{INDENT}{}{}\n);",
            self.name,
            self.fields
                .iter()
                .map(|x| {
                    let sqlitized_field = x.sqlitize();
                    match &self.primary_key {
                        Some(pk) => {
                            if x.name.eq(pk) {
                                format!("{sqlitized_field} PRIMARY KEY")
                            } else {
                                sqlitized_field
                            }
                        }
                        None => sqlitized_field,
                    }
                })
                .collect::<Vec<String>>()
                .join(&format!(",\n{INDENT}")),
            uniqueness_constraint_addition
        )
    }

    fn create_from_row_function(
        table_name_ident: &Ident,
        field_type_pairs: &[(Ident, Type)],
    ) -> TokenStream {
        let function_params = field_type_pairs
            .iter()
            .map(|(field, _)| {
                let ident_str = field.to_string();
                quote! {
                    #field: row.get(#ident_str),
                }
            })
            .collect::<Vec<TokenStream>>();

        quote! {
            pub fn from_row(row: &Row) -> #table_name_ident {
                #table_name_ident {
                    #(#function_params)*
                }
            }
        }
    }

    fn create_new_function(
        table_name_ident: &Ident,
        field_type_pairs: &[(Ident, Type)],
    ) -> TokenStream {
        let function_params = field_type_pairs
            .iter()
            .map(|(field, typ)| {
                quote! {
                    #field: #typ
                }
            })
            .collect::<Vec<TokenStream>>();

        let struct_init_fields = field_type_pairs
            .iter()
            .map(|(field, _)| {
                quote! {
                    #field,
                }
            })
            .collect::<Vec<TokenStream>>();

        quote! {
            pub fn new(#(#function_params),*) -> #table_name_ident {
                #table_name_ident {
                    #(#struct_init_fields)*
                }
            }
        }
    }

    fn create_insert_function(&self) -> TokenStream {
        let composite_uniqueness_check = match &self.composite_uniqueness {
            Some(cu) => {
                let uniqueness_query = format!(
                    "SELECT EXISTS (SELECT 1 FROM {} WHERE {} = $1 AND {} = $2)",
                    &self.name, cu.field1, cu.field2
                );
                let field1 = format_ident!("{}", &cu.field1);
                let field2 = format_ident!("{}", &cu.field2);
                let query_failure_error_message = format!(
                    "Unable to query existence of fields {} = {{}} and {} = {{}} from table {}: {{}}",
                    cu.field1, cu.field2, &self.name
                );
                let get_row_error_message = format!(
                    "Unable to get row value when checking uniqueness constraint for {} and {}: {{}}",
                    cu.field1, cu.field2
                );
                let constraint_check_error_message = format!(
                    "A row already exists where {} = `{{}}` and {} = `{{}}`",
                    cu.field1, cu.field2
                );
                quote! {
                    let row = client
                        .query_one(#uniqueness_query, &[&self.#field1, &self.#field2])
                        .await.map_err(|e| {
                            (false, format!(#query_failure_error_message, &self.#field1, &self.#field2, e))
                        })?;

                    let exists: bool = match row.try_get(0)  {
                        Ok(b) => b,
                        Err(e) => return Err((false, format!(#get_row_error_message, e)))
                    };

                    if exists {
                        return Err((true, format!(#constraint_check_error_message, &self.#field1, &self.#field2)))
                    }
                }
            }
            None => quote! {},
        };

        let field_list_non_optional = self
            .fields
            .iter()
            .filter(|x| !matches!(x.typ, FieldType::AutoIncrementing))
            .collect::<Vec<&Field>>();

        let field_values = field_list_non_optional
            .iter()
            .map(|field| {
                let field_ident = format_ident!("{}", field.name);
                quote! { &self.#field_ident }
            })
            .collect::<Vec<proc_macro2::TokenStream>>();

        let placeholders = field_list_non_optional
            .iter()
            .enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect::<Vec<String>>()
            .join(", ");

        let field_names_str_list = field_list_non_optional
            .iter()
            .map(|f| f.name.as_ref())
            .collect::<Vec<&str>>()
            .join(", ");

        let sql_statement = format!(
            "insert into {} ({field_names_str_list}) values ({placeholders})",
            &self.name
        );
        let sql_statement_str_name_ident = format_ident!("{}", "insert_statement");
        let error_inserting_message = format!(
            "Unable to insert self: {{self:#?}} into table {}",
            &self.name
        );

        quote! {
            pub async fn insert_self_into_table(&self, client: &Client) -> Result<(), (bool, String)> {
                #composite_uniqueness_check

                let #sql_statement_str_name_ident = #sql_statement;

                let insert_result = client.execute(
                    #sql_statement_str_name_ident,
                    &[#(#field_values),*],
                ).await;

                match insert_result {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        Err((false, format!(#error_inserting_message)))
                    }
                }
            }
        }
    }

    fn generate_utility_functions(
        &self,
        table_name_ident: &Ident,
        field_type_pairs: &[(Ident, Type)],
    ) -> TokenStream {
        let new_function = Self::create_new_function(table_name_ident, field_type_pairs);
        let insert_function = self.create_insert_function();
        let from_row_function = Self::create_from_row_function(table_name_ident, field_type_pairs);

        quote! {
            #new_function
            #insert_function
            #from_row_function
        }
    }

    fn structitize(&self) -> TokenStream {
        let table_name_ident = format_ident!("{}", self.name);

        let field_type_pairs = self
            .fields
            .iter()
            .map(|field| {
                let field_name_ident = format_ident!("{}", field.name);

                let typ_raw = field.typ.structitize();

                let typ_raw_optionally_optional = if field.nullable {
                    format!("Option<{typ_raw}>")
                } else {
                    typ_raw
                };

                let proc_macro_type = parse_str::<Type>(&typ_raw_optionally_optional).unwrap();
                (field_name_ident, proc_macro_type)
            })
            .collect::<Vec<(Ident, Type)>>();

        let struct_fields = field_type_pairs
            .iter()
            .map(|(field, typ)| {
                quote! {
                    #field: #typ,
                }
            })
            .collect::<Vec<TokenStream>>();

        let utility_functions =
            self.generate_utility_functions(&table_name_ident, &field_type_pairs);

        quote! {
            #[derive(Debug, Getter)]
            pub struct #table_name_ident {
                #(#struct_fields)*
            }
            impl #table_name_ident {
                #utility_functions
            }
        }
    }
}

struct FieldReference {
    referenced_table: String,
    referenced_field_in_that_table: String,
}
impl FieldReference {
    fn sqlitize(&self) -> String {
        format!(
            "REFERENCES {}({})",
            self.referenced_table, self.referenced_field_in_that_table
        )
    }
}

pub struct Field {
    name: String,
    typ: FieldType,
    nullable: bool,
    unique: bool,
    reference: Option<FieldReference>,
}
impl Field {
    pub fn unique_string_variable_length(field_name: &str) -> Field {
        Field {
            name: field_name.to_string(),
            typ: FieldType::StringVariableLength,
            nullable: false,
            unique: true,
            reference: None,
        }
    }
    pub fn id(field_name: &str) -> Field {
        Field {
            name: field_name.to_string(),
            typ: FieldType::AutoIncrementing,
            nullable: false,
            unique: true,
            reference: None,
        }
    }
    fn geospatial(field_name: &str) -> Field {
        Field {
            name: field_name.to_string(),
            typ: FieldType::DecimalField(DecimalBounds {
                total_digits: 9,
                digits_after_decimal: 6,
            }),
            nullable: false,
            unique: false,
            reference: None,
        }
    }
    pub fn date_time(name: &str) -> Field {
        Field {
            name: name.to_string(),
            typ: FieldType::DateTime,
            nullable: false,
            unique: false,
            reference: None,
        }
    }
    fn string_list(name: &str, unique: bool, nullable: bool) -> Field {
        Field {
            name: name.to_string(),
            typ: FieldType::List(Box::new(FieldType::StringVariableLength)),
            nullable,
            unique,
            reference: None,
        }
    }

    fn sqlitize(&self) -> String {
        let mut qualifiers = vec![self.name.clone()];

        let sqlitized_field_type = self.typ.sqlitize();
        qualifiers.push(sqlitized_field_type);

        if !self.nullable {
            qualifiers.push("NOT NULL".to_string());
        }

        if self.unique {
            qualifiers.push("UNIQUE".to_string());
        }

        if self.reference.is_some() {
            qualifiers.push(self.reference.as_ref().unwrap().sqlitize());
        }

        qualifiers.join(" ")
    }
}

struct DecimalBounds {
    total_digits: u32,
    digits_after_decimal: u32,
}

enum FieldType {
    DecimalField(DecimalBounds),
    Int,
    AutoIncrementing,
    StringVariableLength,
    DateTime,
    ByteArray,
    List(Box<FieldType>),
}
impl FieldType {
    fn structitize(&self) -> String {
        match self {
            FieldType::DecimalField(_) => "Decimal".to_string(),
            FieldType::AutoIncrementing => "Option<u32>".to_string(),
            FieldType::StringVariableLength => "String".to_string(),
            FieldType::Int => "i32".to_string(),
            FieldType::DateTime => "DateTime<Utc>".to_string(),
            FieldType::ByteArray => "Vec<u8>".to_string(),
            FieldType::List(inner_field_type) => format!("Vec<{}>", inner_field_type.structitize()),
        }
    }
    fn sqlitize(&self) -> String {
        match self {
            FieldType::DecimalField(decimal_fields) => format!(
                "DECIMAL({0}, {1})",
                decimal_fields.total_digits, decimal_fields.digits_after_decimal
            ),
            FieldType::AutoIncrementing => "SERIAL".to_string(),
            FieldType::StringVariableLength => "TEXT".to_string(),
            FieldType::Int => "INT".to_string(),
            FieldType::DateTime => "TIMESTAMPTZ".to_string(),
            FieldType::ByteArray => "bytea".to_string(),
            FieldType::List(inner_field_type) => format!("{}[]", inner_field_type.sqlitize()),
        }
    }
}
