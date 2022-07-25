use std::env;
use std::error::Error;
use std::io::{self, BufRead, BufReader};

use encoding_rs::WINDOWS_1252;
use encoding_rs_io::{DecodeReaderBytes, DecodeReaderBytesBuilder};
use rusqlite::{Connection, OpenFlags, Result};
use serde::Deserialize;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
struct CsvRecord {
    #[serde(deserialize_with = "naive_date_from_str")]
    accounting_date: NaiveDate,
    #[serde(deserialize_with = "naive_date_from_str")]
    _availability_date: NaiveDate,
    party: String,
    kind: String,
    comment: String,
    balance: String,
    _balance_currency: String,
    sum: String,
    _sum_currency: String,
}

fn naive_date_from_str<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, "%d.%m.%Y").map_err(serde::de::Error::custom)
}

#[derive(Debug)]
struct Record {
    date: String,
    party: String,
    kind: String,
    comment: String,
    balance: f64,
    sum: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("needs at least one argument for sqlite databases");
    }

    let sqlite_db = &args[1];
    let conn = Connection::open_with_flags(sqlite_db, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .expect("failed to open database connection");

    init_sqlite(&conn).expect("failed to init sqlite table");

    for result in build_csv_reader(io::stdin()).deserialize() {
        let record: CsvRecord = result.expect("failed to parse CSV record");
        process_record(&record, &conn).expect("failed to process record");
    }

    Ok(())
}

fn init_sqlite(conn: &Connection) -> Result<usize> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS transactions (
                  date          TEXT NOT NULL,
                  party         TEXT NOT NULL,
                  kind          TEXT NOT NULL,
                  comment       TEXT NOT NULL,
                  balance       REAL NOT NULL,
                  sum           REAL NOT NULL,
                  PRIMARY KEY (date, party, comment)
        )",
        [],
    )
}

fn build_csv_reader<R: io::Read>(
    reader: R,
) -> csv::Reader<BufReader<DecodeReaderBytes<R, Vec<u8>>>> {
    // build a decoder to use latin1 encoding because that is what
    // ING provides their csv files in.
    let decoder = DecodeReaderBytesBuilder::new()
        .encoding(Some(WINDOWS_1252))
        .build(reader);

    // drop the first 15 lines as they contain some metadata that we do not
    // want to process as CSV.
    let mut buf = io::BufReader::new(decoder);
    for _ in 1..15 {
        let mut string = String::new();
        match buf.read_line(&mut string) {
            Err(err) => panic!("failed to discard first 15 lines of file: {}", err),
            Ok(_) => (),
        };
    }

    // build a csv reader, but ignore headers because there is a duplicate
    // header called 'Währung' and serde really doesn't like that.
    csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b';')
        .from_reader(buf)
}

fn process_record(csv_record: &CsvRecord, conn: &Connection) -> Result<usize> {
    let record = convert_record(csv_record);
    save_to_sqlite(&record, conn)
}

fn convert_record(csv_record: &CsvRecord) -> Record {
    Record {
        date: csv_record.accounting_date.to_string(),
        kind: String::from(&csv_record.kind),
        party: String::from(&csv_record.party),
        comment: String::from(&csv_record.comment),
        balance: match csv_record
            .balance
            .replace(".", "")
            .replace(",", ".")
            .parse::<f64>()
        {
            Ok(val) => val,
            Err(err) => panic!(
                "failed to parse balance '{}' as f64: {}",
                csv_record.balance, err
            ),
        },
        sum: match csv_record
            .sum
            .replace(".", "")
            .replace(",", ".")
            .parse::<f64>()
        {
            Ok(val) => val,
            Err(err) => panic!("failed to parse sum '{}' as f64: {}", csv_record.sum, err),
        },
    }
}

fn save_to_sqlite(record: &Record, conn: &Connection) -> Result<usize> {
    conn.execute(
        "INSERT OR IGNORE INTO transactions (date, kind, party, comment, balance, sum)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        &[
            &record.date,
            &record.kind,
            &record.party,
            &record.comment,
            &record.balance.to_string(),
            &record.sum.to_string(),
        ],
    )
}
