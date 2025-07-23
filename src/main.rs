use schema_and_dao::generate_sql_schema;

fn main() {
    generate_sql_schema!();

    // // Connect to the database
    // let pg_config = Config::from_str(CONNECTION_STRING).unwrap();
    // let mgr_config = ManagerConfig {
    //    recycling_method: RecyclingMethod::Fast,
    // };
    // let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
    // let pool = Pool::builder(mgr).max_size(16).build().unwrap(),
    // let db_client = self.pool.get().await.unwrap()


    // // initialize the database with the sql schema
    // db_client.execute(DB_SCHEMA_SQL).unwrap();

    // // Insert one user
    // let now_utc = Utc::now();
    // let user = User::new(None, "Mike".to_string(), now_utc);
    // user.insert_self_into_table(&db_client);

    // let user_id = db_client.execute("SELECT user_id from User where username = Mike");

    // InterestForUser::new(user_id, "Hockey").insert_self_into_table(&db_client);
    // InterestForUser::new(user_id, "Coding").insert_self_into_table(&db_client);

    // let interests: Vec<String> = InterestForUser::getInterests(user_id, &db_client);
    // println!("{interests}");

    // let retreived_user = User::get_from_table(user_id, &db_client);
    // retreived_user.delete_self_from_table(&db_client);
}
