use std::{fmt, io::{BufRead, BufReader, Read}};

#[repr(align(64))]
pub struct Matrix {
    rows: Vec<usize>,
    cols: Vec<usize>,
    vals: MatrixData,
    nrows: usize,
    ncols: usize,
    nvals: usize,
}

#[cfg(not(feature = "x32"))]
#[repr(align(64))]
enum MatrixData {
    Real(Vec<f64>),
    Complex(Vec<f64>, Vec<f64>),
    Integer(Vec<i64>),
    Binary(),
}

#[cfg(feature = "x32")]
#[repr(align(64))]
enum MatrixData {
    Real(Vec<f32>),
    Complex(Vec<f32>, Vec<f32>),
    Integer(Vec<i32>),
    Binary(),
}

#[derive(Copy, Clone, Debug)]
#[derive(clap::ValueEnum)]
pub enum DataType {
    Real,
    Complex,
    Integer,
    Binary,
}

impl Matrix {
    #[inline]
    pub fn from_reader<R: Read>(rdr: &mut BufReader<R>, data_type: DataType) -> Self {
        let mut lines = rdr.lines()
            .map_while(Result::ok)
            .skip_while(|line| line.starts_with('%'));
            // If comments can appear anywhere, and not just at the start, we should use a filter instead
            //.filter(|line| !line.starts_with('%'));

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
    pub fn sort_row_major(&mut self) {
        let mut permutation: Vec<_> = (0..self.nvals).collect();
        permutation.sort_unstable_by(|&a, &b|
            (self.rows[a], self.cols[a]).cmp(&(self.rows[b], self.cols[b])));
        self.apply(permutation);
    }

    #[inline]
    pub fn sort_col_major(&mut self) {
        let mut permutation: Vec<_> = (0..self.nvals).collect();
        permutation.sort_unstable_by(|&a, &b|
            (self.cols[a], self.rows[a]).cmp(&(self.cols[b], self.rows[b])));
        self.apply(permutation);
    }

    #[inline]
    fn apply(&mut self, mut permutation: Vec<usize>) {
        for i in 0..self.nvals {
            if is_visited(permutation[i]) {
                continue;
            }

            let mut j = i;
            let mut j_idx = permutation[i];

            // When we loop back to the first index, we stop
            while i != j_idx {
                permutation[j] = mark_visited(j_idx);
                self.swap(j, j_idx);
                j = j_idx;
                j_idx = permutation[j];
            }

            permutation[j] = mark_visited(j_idx);
        }
    }

    #[inline]
    fn swap(&mut self, a: usize, b: usize) {
        self.rows.swap(a, b);
        self.cols.swap(a, b);
        match &mut self.vals {
            MatrixData::Real(xs) => {
                xs.swap(a, b);
            },
            MatrixData::Complex(xs, ys) => {
                xs.swap(a, b);
                ys.swap(a, b);
            },
            MatrixData::Integer(xs) => {
                xs.swap(a, b);
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
        use DataType::*;
        match data_type {
            Real => MatrixData::Real(Vec::new()),
            Complex => MatrixData::Complex(Vec::new(), Vec::new()),
            Integer => MatrixData::Integer(Vec::new()),
            Binary => MatrixData::Binary(),
        }
    }

    #[inline]
    fn with_capacity(data_type: DataType, nvals: usize) -> Self {
        use DataType::*;
        match data_type {
            Real => MatrixData::Real(Vec::with_capacity(nvals)),
            Complex => MatrixData::Complex(Vec::with_capacity(nvals), Vec::with_capacity(nvals)),
            Integer => MatrixData::Integer(Vec::with_capacity(nvals)),
            Binary => MatrixData::Binary(),
        }
    }
}

impl fmt::Debug for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = f.width().unwrap_or(5);
        let p = f.precision().unwrap_or(2);

        let name = if n >= self.nvals { "Matrix" } else { &format!("Matrix (head={n})") };
        let mut wtr = f.debug_struct(name);
        wtr.field("nrows", &self.nrows)
            .field("ncols", &self.ncols)
            .field("nvals", &self.nvals)
            .field("rows", &format_args!("{:?}", &self.rows[..n]))
            .field("cols", &format_args!("{:?}", &self.cols[..n]));

        match &self.vals {
            MatrixData::Real(xs) => {
                wtr.field("real", &format_args!("{1:.*?}", p, &xs[..n]));
            },
            MatrixData::Complex(xs, ys) => {
                wtr.field("real", &format_args!("{1:.*?}", p, &xs[..n]));
                wtr.field("imag", &format_args!("{1:.*?}", p, &ys[..n]));
            },
            MatrixData::Integer(xs) => {
                wtr.field("int", &format_args!("{:?}", &xs[..n]));
            },
            MatrixData::Binary() => {
                /* nothing to do */
            },
        }

        wtr.finish()
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {} {}", self.nrows, self.ncols, self.nvals)?;
        (0..self.nvals).try_for_each(|i| {
            use MatrixData::*;
            match &self.vals {
                Real(xs) => writeln!(f, "{} {} {}", self.rows[i], self.cols[i], xs[i]),
                Complex(xs, ys) => writeln!(f, "{} {} {} {}", self.rows[i], self.cols[i], xs[i], ys[i]),
                Integer(xs) => writeln!(f, "{} {} {}", self.rows[i], self.cols[i], xs[i]),
                Binary() => writeln!(f, "{} {}", self.rows[i], self.cols[i]),
            }
        })
    }
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

/// Mark the element at this index as visited by toggling the most-significant bit.
#[inline(always)]
fn mark_visited(idx: usize) -> usize {
    const MASK: usize = isize::MIN as usize;
    idx ^ MASK
}

/// Check if the element at this index has been visited by reading the most-significant bit.
#[inline(always)]
fn is_visited(idx: usize) -> bool {
    const MASK: usize = isize::MIN as usize;
    (idx & MASK) != 0
}
