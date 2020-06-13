use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

mod helper;

use gluesql::{execute, Payload, Row, RowError, SledStorage, Value, ValueError};

fn new(path: &str) -> SledStorage {
    match std::fs::remove_dir_all(path) {
        Ok(()) => (),
        Err(e) => {
            println!("fs::remove_file {:?}", e);
        }
    }

    SledStorage::new(path.to_owned()).expect("SledStorage::new")
}

#[test]
fn migrate() {
    let storage = new("data.db");
    let dialect = GenericDialect {};
    let run = |sql| {
        let ast = Parser::parse_sql(&dialect, sql).unwrap();
        let query = &ast[0];

        execute(&storage, &query)
    };

    let sql = r#"
CREATE TABLE Test (
    id INT,
    num INT,
    name TEXT
)"#;

    run(sql).unwrap();

    let sqls = [
        "INSERT INTO Test (id, num, name) VALUES (1, 2, \"Hello\")",
        "INSERT INTO Test (id, num, name) VALUES (1, 9, \"World\")",
        "INSERT INTO Test (id, num, name) VALUES (3, 4, \"Great\")",
    ];

    sqls.iter().for_each(|sql| {
        run(sql).unwrap();
    });

    let error_cases = vec![
        (
            RowError::UnsupportedAstValueType.into(),
            "INSERT INTO Test (id, num) VALUES (3 * 2, 1);",
        ),
        (
            RowError::MultiRowInsertNotSupported.into(),
            "INSERT INTO Test (id, num) VALUES (1, 1), (2, 1);",
        ),
        (
            ValueError::FailedToParseNumber.into(),
            "INSERT INTO Test (id, num) VALUES (1.1, 1);",
        ),
    ];

    error_cases.into_iter().for_each(|(error, sql)| {
        let error = Err(error);

        assert_eq!(error, run(sql));
    });

    use Value::*;

    let found = run("SELECT id, num, name FROM Test").expect("select");
    let expected = select!(
        I64 I64 String;
        1   2   "Hello".to_owned();
        1   9   "World".to_owned();
        3   4   "Great".to_owned()
    );
    assert_eq!(expected, found);

    let found = run("SELECT id, num, name FROM Test WHERE id = 1").expect("select");
    let expected = select!(
        I64 I64 String;
        1   2   "Hello".to_owned();
        1   9   "World".to_owned()
    );
    assert_eq!(expected, found);

    /*
    helper.run_and_print("UPDATE Test SET id = 2");

    let found = helper.run("SELECT id FROM Test").expect("select");
    let expected = select!(I64; 2; 2; 2);
    assert_eq!(expected, found);

    let found = helper.run("SELECT id, num FROM Test").expect("select");
    let expected = select!(I64 I64; 2 2; 2 9; 2 4);
    assert_eq!(expected, found);
    */
}
