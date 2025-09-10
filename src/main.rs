use std::{
    fmt,
    fs::File,
    io::{self, BufReader, BufWriter, Write},
    path::PathBuf,
    time::Instant,
};

use clap::Parser;
use matrix_market_transform::*;

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

#[derive(Copy, Clone, Debug)]
#[derive(clap::ValueEnum)]
pub enum SortOrder {
    RowMajor,
    ColMajor,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SortOrder::*;
        match self {
            RowMajor => write!(f, "row-major"),
            ColMajor => write!(f, "col-major"),
        }
    }
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

    let now = Instant::now();
    let mut m = Matrix::from_reader(&mut rdr, data_type);
    println!("Read: {:?}", now.elapsed());
    println!("{:#?}", m);

    let now = Instant::now();
    match sort_order {
        SortOrder::RowMajor => m.sort_row_major(),
        SortOrder::ColMajor => m.sort_col_major(),
    }
    println!("Sort: {:?}", now.elapsed());
    println!("{:#?}", m);

    if let Some(path) = output_file {
        let file = File::create(path)?;
        let mut wtr = BufWriter::new(file);

        let now = Instant::now();
        write!(wtr, "{}", m)?;
        println!("Write: {:?}", now.elapsed());
    }

    Ok(())
}
