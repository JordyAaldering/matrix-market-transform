use std::{fmt, fs, io::{BufRead, BufReader, Read}, str};

use memmap2::MmapOptions;
use rayon::prelude::*;

#[repr(align(64))]
pub struct Matrix {
    rows: Vec<usize>,
    cols: Vec<usize>,
    vals: MatrixData,
    nrows: usize,
    ncols: usize,
    nvals: usize,
}

#[cfg(not(feature = "x64"))]
#[repr(align(64))]
enum MatrixData {
    Real(Vec<f32>),
    Complex(Vec<f32>, Vec<f32>),
    Integer(Vec<i32>),
    Bool(),
}

#[cfg(feature = "x64")]
#[repr(align(64))]
enum MatrixData {
    Real(Vec<f64>),
    Complex(Vec<f64>, Vec<f64>),
    Integer(Vec<i64>),
    Bool(),
}

#[derive(Copy, Clone, Debug)]
#[derive(clap::ValueEnum)]
pub enum DataType {
    Real,
    Complex,
    Integer,
    Bool,
}

impl Matrix {
    pub fn from_mmap(file: fs::File, data_type: DataType) -> Self {
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        let mut lines = mmap.split(|&b| b == b'\n')
            // We deliberately do not `map` yet because we are still in sequential mode
            .skip_while(|b| b.trim_ascii()[0] == b'%');

        if let Some(header) = lines.next() {
            let parts: Vec<_> = header.split(|&b| b.is_ascii_whitespace()).collect();
            let nrows = str::from_utf8(parts[0]).unwrap().parse().unwrap();
            let ncols = str::from_utf8(parts[1]).unwrap().parse().unwrap();
            let nvals = str::from_utf8(parts[2]).unwrap().parse().unwrap();

            let mut rows = vec![0usize; nvals];
            let mut cols = vec![0usize; nvals];

            let lines: Vec<_> = lines.collect();
            let vals = match data_type {
                DataType::Real => {
                    let mut xs = vec![0.0; nvals];
                    lines.into_par_iter()
                        .zip(rows.par_iter_mut())
                        .zip(cols.par_iter_mut())
                        .zip(xs.par_iter_mut())
                        .for_each(|(((line, row), col), x)| {
                            let parts: Vec<_> = line.trim_ascii().split(|&b| b.is_ascii_whitespace()).collect();
                            *row = str::from_utf8(parts[0]).unwrap().parse().unwrap();
                            *col = str::from_utf8(parts[1]).unwrap().parse().unwrap();
                            *x = str::from_utf8(parts[2]).unwrap().parse().unwrap();
                        });
                    MatrixData::Real(xs)
                },
                DataType::Complex => {
                    let mut xs = vec![0.0; nvals];
                    let mut ys = vec![0.0; nvals];
                    lines.into_par_iter()
                        .zip(rows.par_iter_mut())
                        .zip(cols.par_iter_mut())
                        .zip(xs.par_iter_mut())
                        .zip(ys.par_iter_mut())
                        .for_each(|((((line, row), col), x), y)| {
                            let parts: Vec<_> = line.split(|&b| b.is_ascii_whitespace()).collect();
                            *row = str::from_utf8(parts[0]).unwrap().parse().unwrap();
                            *col = str::from_utf8(parts[1]).unwrap().parse().unwrap();
                            *x = str::from_utf8(parts[2]).unwrap().parse().unwrap();
                            *y = str::from_utf8(parts[3]).unwrap().parse().unwrap();
                        });
                    MatrixData::Complex(xs, ys)
                },
                DataType::Integer => {
                    let mut xs = vec![0; nvals];
                    lines.into_par_iter()
                        .zip(rows.par_iter_mut())
                        .zip(cols.par_iter_mut())
                        .zip(xs.par_iter_mut())
                        .for_each(|(((line, row), col), x)| {
                            let parts: Vec<_> = line.split(|&b| b.is_ascii_whitespace()).collect();
                            *row = str::from_utf8(parts[0]).unwrap().parse().unwrap();
                            *col = str::from_utf8(parts[1]).unwrap().parse().unwrap();
                            *x = str::from_utf8(parts[2]).unwrap().parse().unwrap();
                        });
                    MatrixData::Integer(xs)
                },
                DataType::Bool => {
                    lines.into_par_iter()
                        .zip(rows.par_iter_mut())
                        .zip(cols.par_iter_mut())
                        .for_each(|((line, row), col)| {
                            let parts: Vec<_> = line.split(|&b| b.is_ascii_whitespace()).collect();
                            *row = str::from_utf8(parts[0]).unwrap().parse().unwrap();
                            *col = str::from_utf8(parts[1]).unwrap().parse().unwrap();
                        });
                    MatrixData::Bool()
                },
            };

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

    pub fn from_reader<R: Read>(rdr: BufReader<R>, data_type: DataType) -> Self {
        let mut lines = rdr.lines()
            .map_while(Result::ok)
            // We assume comments can only appear at the start of the file
            .skip_while(|line| line.starts_with('%'));

        if let Some(header) = lines.next() {
            let parts: Vec<_> = header.split_ascii_whitespace().collect();
            let nrows = parts[0].parse().unwrap();
            let ncols = parts[1].parse().unwrap();
            let nvals = parts[2].parse().unwrap();

            let mut rows = Vec::with_capacity(nvals);
            let mut cols = Vec::with_capacity(nvals);
            let mut vals = MatrixData::with_capacity(data_type, nvals);

            for line in lines {
                let parts: Vec<_> = line.split_ascii_whitespace().collect();
                rows.push(parts[0].parse().unwrap());
                cols.push(parts[1].parse().unwrap());
                match &mut vals {
                    MatrixData::Real(xs) => {
                        xs.push(parts[2].parse().unwrap())
                    },
                    MatrixData::Complex(xs, ys) => {
                        xs.push(parts[2].parse().unwrap());
                        ys.push(parts[3].parse().unwrap());
                    },
                    MatrixData::Integer(xs) => {
                        xs.push(parts[2].parse().unwrap())
                    },
                    MatrixData::Bool() => {
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

    pub fn sort_row_major(&mut self) {
        match &mut self.vals {
            MatrixData::Real(xs) => {
                let mut zipped: Vec<_> = (0..self.nvals)
                    .map(|i| (self.rows[i], self.cols[i], xs[i]))
                    .collect();

                zipped.par_sort_unstable_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));

                zipped.into_par_iter()
                    .zip(self.rows.par_iter_mut())
                    .zip(self.cols.par_iter_mut())
                    .zip(xs.par_iter_mut())
                    .for_each(|(((e, row), col), x)| {
                        *row = e.0;
                        *col = e.1;
                        *x = e.2;
                    });
            },
            MatrixData::Complex(xs, ys) => {
                let mut zipped: Vec<_> = (0..self.nvals)
                    .map(|i| (self.rows[i], self.cols[i], xs[i], ys[i]))
                    .collect();

                zipped.par_sort_unstable_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));

                zipped.into_par_iter()
                    .zip(self.rows.par_iter_mut())
                    .zip(self.cols.par_iter_mut())
                    .zip(xs.par_iter_mut())
                    .zip(ys.par_iter_mut())
                    .for_each(|((((e, row), col), x), y)| {
                        *row = e.0;
                        *col = e.1;
                        *x = e.2;
                        *y = e.3;
                    });
            },
            MatrixData::Integer(xs) => {
                let mut zipped: Vec<_> = (0..self.nvals)
                    .map(|i| (self.rows[i], self.cols[i], xs[i]))
                    .collect();

                zipped.par_sort_unstable_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));

                zipped.into_par_iter()
                    .zip(self.rows.par_iter_mut())
                    .zip(self.cols.par_iter_mut())
                    .zip(xs.par_iter_mut())
                    .for_each(|(((e, row), col), x)| {
                        *row = e.0;
                        *col = e.1;
                        *x = e.2;
                    });
            },
            MatrixData::Bool() => {
                let mut zipped: Vec<_> = (0..self.nvals)
                    .map(|i| (self.rows[i], self.cols[i]))
                    .collect();

                zipped.par_sort_unstable_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));

                zipped.into_par_iter()
                    .zip(self.rows.par_iter_mut())
                    .zip(self.cols.par_iter_mut())
                    .for_each(|((e, row), col)| {
                        *row = e.0;
                        *col = e.1;
                    });
            },
        };
    }

    pub fn sort_col_major(&mut self) {
        match &mut self.vals {
            MatrixData::Real(xs) => {
                let mut zipped: Vec<_> = (0..self.nvals)
                    .map(|i| (self.rows[i], self.cols[i], xs[i]))
                    .collect();

                zipped.par_sort_unstable_by(|a, b| (a.1, a.0).cmp(&(b.1, b.0)));

                zipped.into_par_iter()
                    .zip(self.rows.par_iter_mut())
                    .zip(self.cols.par_iter_mut())
                    .zip(xs.par_iter_mut())
                    .for_each(|(((e, row), col), x)| {
                        *row = e.0;
                        *col = e.1;
                        *x = e.2;
                    });
            },
            MatrixData::Complex(xs, ys) => {
                let mut zipped: Vec<_> = (0..self.nvals)
                    .map(|i| (self.rows[i], self.cols[i], xs[i], ys[i]))
                    .collect();

                zipped.par_sort_unstable_by(|a, b| (a.1, a.0).cmp(&(b.1, b.0)));

                zipped.into_par_iter()
                    .zip(self.rows.par_iter_mut())
                    .zip(self.cols.par_iter_mut())
                    .zip(xs.par_iter_mut())
                    .zip(ys.par_iter_mut())
                    .for_each(|((((e, row), col), x), y)| {
                        *row = e.0;
                        *col = e.1;
                        *x = e.2;
                        *y = e.3;
                    });
            },
            MatrixData::Integer(xs) => {
                let mut zipped: Vec<_> = (0..self.nvals)
                    .map(|i| (self.rows[i], self.cols[i], xs[i]))
                    .collect();

                zipped.par_sort_unstable_by(|a, b| (a.1, a.0).cmp(&(b.1, b.0)));

                zipped.into_par_iter()
                    .zip(self.rows.par_iter_mut())
                    .zip(self.cols.par_iter_mut())
                    .zip(xs.par_iter_mut())
                    .for_each(|(((e, row), col), x)| {
                        *row = e.0;
                        *col = e.1;
                        *x = e.2;
                    });
            },
            MatrixData::Bool() => {
                let mut zipped: Vec<_> = (0..self.nvals)
                    .map(|i| (self.rows[i], self.cols[i]))
                    .collect();

                zipped.par_sort_unstable_by(|a, b| (a.1, a.0).cmp(&(b.1, b.0)));

                zipped.into_par_iter()
                    .zip(self.rows.par_iter_mut())
                    .zip(self.cols.par_iter_mut())
                    .for_each(|((e, row), col)| {
                        *row = e.0;
                        *col = e.1;
                    });
            },
        };
    }

    /// Slightly more memory-friendly approach to sorting.
    /// Only allocates one additional array of length `nvals`.
    pub fn permute_row_major(&mut self) {
        let mut permutation: Vec<_> = (0..self.nvals).collect();
        permutation.sort_unstable_by(|&a, &b|
            (self.rows[a], self.cols[a]).cmp(&(self.rows[b], self.cols[b])));
        self.apply_permutation(permutation);
    }

    /// Slightly more memory-friendly approach to sorting.
    /// Only allocates one additional array of length `nvals`.
    pub fn permute_col_major(&mut self) {
        let mut permutation: Vec<_> = (0..self.nvals).collect();
        permutation.sort_unstable_by(|&a, &b|
            (self.cols[a], self.rows[a]).cmp(&(self.cols[b], self.rows[b])));
        self.apply_permutation(permutation);
    }

    fn apply_permutation(&mut self, mut permutation: Vec<usize>) {
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
            MatrixData::Bool() => {
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
            Bool => MatrixData::Bool(),
        }
    }

    #[inline]
    fn with_capacity(data_type: DataType, nvals: usize) -> Self {
        use DataType::*;
        match data_type {
            Real => MatrixData::Real(Vec::with_capacity(nvals)),
            Complex => MatrixData::Complex(Vec::with_capacity(nvals), Vec::with_capacity(nvals)),
            Integer => MatrixData::Integer(Vec::with_capacity(nvals)),
            Bool => MatrixData::Bool(),
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
            MatrixData::Bool() => {
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
                Bool() => writeln!(f, "{} {}", self.rows[i], self.cols[i]),
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
            Bool => write!(f, "bool"),
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
