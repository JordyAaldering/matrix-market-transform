mod permutation;

use std::{fmt, io::{BufRead, BufReader, Read}};

#[derive(Copy, Clone, Debug)]
#[derive(clap::ValueEnum)]
pub enum DataType {
    Real,
    Complex,
    Integer,
    Binary,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DataType::*;
        match self {
            Real => write!(f, "real"),
            Complex => write!(f, "complex"),
            Integer => write!(f, "integer"),
            Binary => write!(f, "binary"),
        }
    }
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

#[derive(Clone, Debug)]
pub struct Matrix {
    rows: Vec<usize>,
    cols: Vec<usize>,
    vals: MatrixData,
    nrows: usize,
    ncols: usize,
    nvals: usize,
}

#[cfg(not(feature = "x32"))]
#[derive(Clone, Debug)]
enum MatrixData {
    Real(Vec<f64>),
    Complex(Vec<f64>, Vec<f64>),
    Integer(Vec<i64>),
    Binary(),
}

#[cfg(feature = "x32")]
#[derive(Clone, Debug)]
enum MatrixData {
    Real(Vec<f32>),
    Complex(Vec<f32>, Vec<f32>),
    Integer(Vec<i32>),
    Binary(),
}

impl Matrix {
    #[inline]
    pub fn from_reader<R: Read>(rdr: &mut BufReader<R>, data_type: DataType) -> Self {
        let mut lines = rdr.lines()
            .map_while(Result::ok)
            .filter(|line| !line.starts_with('%'));

        if let Some(header) = lines.next() {
            let mut parts = header.split_ascii_whitespace();
            let nrows = parts.next().unwrap().parse().unwrap();
            let ncols = parts.next().unwrap().parse().unwrap();
            let nvals = parts.next().unwrap().parse().unwrap();

            let mut rows = Vec::with_capacity(nvals);
            let mut cols = Vec::with_capacity(nvals);
            let mut vals = MatrixData::with_capacity(data_type, nvals);

            for line in lines {
                let mut parts = line.split_ascii_whitespace();
                rows.push(parts.next().unwrap().parse().unwrap());
                cols.push(parts.next().unwrap().parse().unwrap());
                match &mut vals {
                    MatrixData::Real(xs) => {
                        xs.push(parts.next().unwrap().parse().unwrap())
                    },
                    MatrixData::Complex(xs, ys) => {
                        xs.push(parts.next().unwrap().parse().unwrap());
                        ys.push(parts.next().unwrap().parse().unwrap());
                    },
                    MatrixData::Integer(xs) => {
                        xs.push(parts.next().unwrap().parse().unwrap())
                    },
                    MatrixData::Binary() => {
                        /* nothing to do */
                    },
                }
            }

            Self { rows, cols, vals, nrows, ncols, nvals }
        } else {
            // File is empty or contains only comments, return empty matrix
            Self {
                rows: Vec::new(),
                cols: Vec::new(),
                vals: MatrixData::new(data_type),
                nrows: 0, ncols: 0, nvals: 0,
            }
        }
    }

    #[inline]
    pub fn sort(&mut self, mode: SortOrder) {
        let mut permutation: Vec<_> = (0..self.nvals).collect();
        // We can use an unstable sort, because no two elements can have the
        // same column and row index, i.e. there are no equal elements.
        match mode {
            SortOrder::RowMajor => {
                permutation.sort_unstable_by(|&a, &b|
                    (self.rows[a], self.cols[a]).cmp(&(self.rows[b], self.cols[b])));
            },
            SortOrder::ColMajor => {
                permutation.sort_unstable_by(|&a, &b|
                    (self.cols[a], self.rows[a]).cmp(&(self.cols[b], self.rows[b])));
            },
        };

        permutation::apply(&mut permutation, &mut self.rows);
        permutation::reset(&mut permutation);
        permutation::apply(&mut permutation, &mut self.cols);
        match &mut self.vals {
            MatrixData::Real(xs) => {
                permutation::reset(&mut permutation);
                permutation::apply(&mut permutation, xs);
            },
            MatrixData::Complex(xs, ys) => {
                permutation::reset(&mut permutation);
                permutation::apply(&mut permutation, xs);
                permutation::reset(&mut permutation);
                permutation::apply(&mut permutation, ys);
            },
            MatrixData::Integer(xs) => {
                permutation::reset(&mut permutation);
                permutation::apply(&mut permutation, xs);
            },
            MatrixData::Binary() => {
                /* nothing to do */
            },
        }
    }
}

impl MatrixData {
    #[inline]
    fn new(data_type: DataType) -> Self {
        use MatrixData::*;
        match data_type {
            DataType::Real => Real(Vec::new()),
            DataType::Complex => Complex(Vec::new(), Vec::new()),
            DataType::Integer => Integer(Vec::new()),
            DataType::Binary => Binary(),
        }
    }

    #[inline]
    fn with_capacity(data_type: DataType, nvals: usize) -> Self {
        use MatrixData::*;
        match data_type {
            DataType::Real => Real(Vec::with_capacity(nvals)),
            DataType::Complex => Complex(Vec::with_capacity(nvals), Vec::with_capacity(nvals)),
            DataType::Integer => Integer(Vec::with_capacity(nvals)),
            DataType::Binary => Binary(),
        }
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {} {}", self.nrows, self.ncols, self.nvals)?;

        for i in 0..self.nvals {
            use MatrixData::*;
            match &self.vals {
                Real(xs) => writeln!(f, "{} {} {}", self.rows[i], self.cols[i], xs[i])?,
                Complex(xs, ys) => writeln!(f, "{} {} {} {}", self.rows[i], self.cols[i], xs[i], ys[i])?,
                Integer(xs) => writeln!(f, "{} {} {}", self.rows[i], self.cols[i], xs[i])?,
                Binary() => writeln!(f, "{} {}", self.rows[i], self.cols[i])?,
            }
        }

        Ok(())
    }
}
