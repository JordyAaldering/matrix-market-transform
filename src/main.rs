use std::{fs::File, io::{self, BufReader, BufWriter, Write}, path::PathBuf};

use clap::Parser;
use row_col_major::*;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    pub input_file: PathBuf,

    #[arg(short('o'))]
    pub output_file: Option<PathBuf>,

    #[arg(short('t'), long("type"), default_value_t = DataType::Real)]
    pub data_type: DataType,

    #[arg(short('s'), long("sort"), default_value_t = SortOrder::RowMajor)]
    pub sort_order: SortOrder,
}

fn main() -> io::Result<()> {
    let Args {
        input_file,
        output_file,
        data_type,
        sort_order,
    } = Args::parse();

    let file = File::open(input_file)?;
    let mut rdr = BufReader::new(file);
    let mut m = Matrix::from_reader(&mut rdr, data_type);
    m.sort(sort_order);

    if let Some(path) = output_file {
        let file = File::create(path)?;
        let mut wtr = BufWriter::new(file);
        write!(wtr, "{}", m)?;
    }

    Ok(())
}
