use std::process::Command;
use std::error::Error;

#[derive(Debug)]
struct Column {
    table_name: String,
    column_name: String,
    datatype: String,
    column_type: String,
    collation_name: String,
    not_null: bool,
    column_length: i32,
}

fn find_latin1_fields() -> Result<Vec<Column>, Box<dyn Error>> {
    let locate_output =
        Command::new("mysql")
            .args(&["-u", "root",
                "-pTHE_PASSWORD",
                "--skip-column-names",
                "-e", "SELECT * FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_SCHEMA = 'mwdb' AND COLLATION_NAME LIKE 'latin1%'"])
            .output()?
            .stdout
        ;
    Ok(String::from_utf8(locate_output)?
        .split("\n")
        // there's an empty line at the empty
        .filter(|row_string| row_string.len() > 0)
        .map(|row_string| {
            let row: Vec<&str> = row_string.split("\t").collect();
            Column {
                table_name: String::from(row[2]),
                column_name: String::from(row[3]),
                datatype: String::from(row[7]),
                column_type: String::from(row[15]),
                collation_name: String::from(row[14]),
                not_null: row[6] == "NO",
                column_length: row[8].parse().unwrap(),
            }
        })
        .collect())
}

fn main() {
    let latin1_fields =
        find_latin1_fields()
            .expect("Failed to find fields");
    latin1_fields
        .into_iter()
        .for_each(|column| {
            // if ("varchar" == column.datatype || "char" == column.datatype || "text" == column.datatype || "enum" == column.datatype || "mediumtext" == column.datatype) {} else {
            // }
            // println!("Column {:?}", column)
            let not_null = if column.not_null { "NOT NULL" } else { "" };
            let intermediate_type;
            let collation =
                if column.collation_name.contains("_bin") { "UTF8MB4_GENERAL_CI" } else { "UTF8MB4_GENERAL_CI" };
            if "mediumtext" == column.datatype {
                intermediate_type = String::from("MEDIUMBLOB");
            }
            else if "varchar" == column.datatype {
                intermediate_type = format!("VARBINARY({})", column.column_length);
            }
            else {
                return
            }
            println!("ALTER TABLE {} MODIFY {} {} {};",
                     column.table_name, column.column_name, intermediate_type, not_null);
            println!("ALTER TABLE {} MODIFY {} {} {} COLLATE UTF8MB4_GENERAL_CI;",
                     column.table_name, column.column_name, column.column_type, not_null);
        })
}
