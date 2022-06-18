use std::io::{self, BufRead, BufReader};
use std::error::Error;

use serde::Deserialize;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::{DecodeReaderBytesBuilder,DecodeReaderBytes};

#[derive(Debug, Deserialize)]
struct CsvRecord {
    _accounting_date: String,
    availability_date: String,
    party: String,
    kind: String,
    comment: String,
    balance: String,
    _balance_currency: String,
    sum: String,
    _sum_currency: String,
}

#[derive(Debug)]
struct Record {
    id: i32,
    date: String,
    party: String,
    kind: String,
    comment: String,
    balance: String,
    sum: String,
}


fn main() -> Result<(), Box<dyn Error>> {
    for result in build_csv_reader(io::stdin()).deserialize() {
        let record: CsvRecord = result?;
        process_record(&record)?;
    };

    Ok(())
}

fn build_csv_reader<R: io::Read>(reader: R) -> csv::Reader<BufReader<DecodeReaderBytes<R, Vec<u8>>>> {
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
    };

    // build a csv reader, but ignore headers because there is a duplicate
    // header called 'Währung' and serde really doesn't like that.
    csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b';')
        .from_reader(buf)
}

fn process_record(csv_record: &CsvRecord) -> Result<(), Box<dyn Error>> {
    let record = convert_record(csv_record);
    println!("{:?}", record);
    Ok(())
}

fn convert_record(csv_record: &CsvRecord) -> Record {
    Record{
        id: 0,
        date: String::from(&csv_record.availability_date),
        kind: String::from(&csv_record.kind),
        party: String::from(&csv_record.party),
        comment: String::from(&csv_record.comment),
        balance: String::from(&csv_record.balance),
        sum: String::from(&csv_record.sum),
    }
}
