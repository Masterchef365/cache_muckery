use rand::distributions::Uniform;
use rand::prelude::*;
use std::time::Instant;
use std::collections::HashMap;

fn main() {
    let mut args = std::env::args().skip(1);

    let usage = "Usage: cache_fuckery <rows> <cols> <n_operations>";
    let rows: usize = args.next().unwrap_or("100000".into()).parse().expect(usage); // Number of rows
    let cols: usize = args.next().unwrap_or("100".into()).parse().expect(usage); // Number of columns
    let n_adds: usize = args.next().unwrap_or("400".into()).parse().expect(usage); // Number of adds

    println!("# of rows: {}", rows);
    println!("# of columns: {}", cols);
    println!("# of operations: {}", n_adds);

    let mut rng = rand::thread_rng();

    // Synthesize data
    let uniform = Uniform::new(0., 1.);
    let data: Vec<f32> = uniform.sample_iter(&mut rng).take(rows * cols).collect();

    // Synthesize add operations
    let uniform = Uniform::new(0, cols);
    let adds: Vec<(usize, usize)> = (0..n_adds)
        .map(|_| (uniform.sample(&mut rng), uniform.sample(&mut rng)))
        .collect();

    // Run assessment of rows perf
    println!("Rows:");
    let start_time = Instant::now();
    let mut by_rows_data = data.clone();
    by_rows(&mut by_rows_data, &adds, cols);
    println!("\tTime: {}s", start_time.elapsed().as_secs_f32());

    // Run assessment of transposed rows perf
    println!("Rows (transposed)");
    let start_time = Instant::now();
    let mut by_rows_transposed_data = data.clone();
    by_rows_transposed(&mut by_rows_transposed_data, &adds, cols);
    println!("\tTime: {}s", start_time.elapsed().as_secs_f32());

    // Run assessment of cols perf
    println!("Cols:");
    let start_time = Instant::now();
    let mut by_cols_data = data.clone();
    by_cols(&mut by_cols_data, &adds, cols);
    println!("\tTime: {}s", start_time.elapsed().as_secs_f32());

    // Run assessment of smart cols perf
    println!("Cols (smart)");
    let start_time = Instant::now();
    let mut by_cols_smart_data = data.clone();
    by_cols_smart(&mut by_cols_smart_data, &adds, cols);
    println!("\tTime: {}s", start_time.elapsed().as_secs_f32());

    println!("Verifying solutions...");
    assert_eq!(&by_rows_data, &by_cols_data);
    assert_eq!(&by_rows_data, &by_cols_smart_data);
    assert_eq!(&by_rows_data, &by_rows_transposed_data);
    println!("Finished!");
}

/// Rows on the outside, adds on the inside
fn by_rows(data: &mut [f32], adds: &[(usize, usize)], n_cols: usize) {
    for row in data.chunks_exact_mut(n_cols) {
        for &(column_a, column_b) in adds {
            row[column_a] += row[column_b];
        }
    }
}

/// Rows on the inside, adds on the outside
fn by_rows_transposed(data: &mut [f32], adds: &[(usize, usize)], n_cols: usize) {
    for &(column_a, column_b) in adds {
        for row in data.chunks_exact_mut(n_cols) {
            row[column_a] += row[column_b];
        }
    }
}

/// Transpose into columns first
fn by_cols(data: &mut [f32], adds: &[(usize, usize)], n_cols: usize) {
    //  Split into columns
    let mut cols: Vec<Vec<f32>> = vec![vec![]; n_cols];
    for row in data.chunks_exact(n_cols) {
        for (col_idx, element) in row.iter().enumerate() {
            cols[col_idx].push(*element);
        }
    }

    // Add columns together
    let start_time = Instant::now();
    let n_rows = data.len() / n_cols;
    for &(column_a, column_b) in adds {
        let mut new_col = Vec::with_capacity(n_rows);
        for (elem_a, elem_b) in cols[column_a].iter().zip(&cols[column_b]) {
            new_col.push(elem_a + elem_b);
        }
        cols[column_a] = new_col;
    }
    println!("\tAdding time inside columns: {}s", start_time.elapsed().as_secs_f32());

    // Merge columns back into rows
    for (row_idx, row) in data.chunks_exact_mut(n_cols).enumerate() {
        for (data_elem, col) in row.iter_mut().zip(&cols) {
            *data_elem = col[row_idx];
        }
    }
}

/// Transpose into columns first (but only the columns you need)
fn by_cols_smart(data: &mut [f32], adds: &[(usize, usize)], n_cols: usize) {
    //  Split into columns, only those which appear in the ads
    let mut cols: HashMap<usize, Vec<f32>> = HashMap::new();
    for &(a, b) in adds {
        cols.entry(a).or_insert_with(|| vec![]);
        cols.entry(b).or_insert_with(|| vec![]);
    }
    
    for (column_idx, column) in &mut cols {
        for row in data.chunks_exact(n_cols) {
            column.push(row[*column_idx]);
        }
    }

    // Add columns together
    let start_time = Instant::now();
    let n_rows = data.len() / n_cols;
    for (column_a, column_b) in adds {
        let mut new_col = Vec::with_capacity(n_rows);
        for (elem_a, elem_b) in cols[column_a].iter().zip(&cols[column_b]) {
            new_col.push(elem_a + elem_b);
        }
        *cols.get_mut(column_a).unwrap() = new_col;
    }
    println!("\tAdding time inside columns (smart): {}s", start_time.elapsed().as_secs_f32());

    // Merge columns back into rows
    for (row_idx, row) in data.chunks_exact_mut(n_cols).enumerate() {
        for (col_idx, column) in &cols {
            row[*col_idx] = column[row_idx];
        }
    }
}

