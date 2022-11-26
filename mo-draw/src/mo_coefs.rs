use ndarray::{ Array2 };
use crate::error::Error;

fn argvs_shape(argvs: &Vec<Vec<String>>) -> Result<(usize, usize), Error> {
    let height = argvs.len();
    let width = argvs.first()
                     .unwrap_or(&vec![])
                     .len();

    for argv in &argvs[1..] {
        if argv.len() != width {
            return Err("inconsistent argvs shape".into());
        }
    }

    Ok((height, width))
}

pub fn mo_coefs_from_argvs(argvs: &Vec<Vec<String>>) -> Result<Array2<f64>, Error> {
    let shape = argvs_shape(argvs)?;
    let mut mo_coefs = Array2::zeros(shape);

    // copy has to happen in this scope so we can propogate parse errors
    for i in 0..(shape.0) {
        for j in 0..(shape.1) {
            mo_coefs[(i, j)] = argvs[i][j].parse()?;
        }
    }

    Ok(mo_coefs)
}

fn find_shape<T>(elems: &Vec<Vec<T>>) -> Result<(usize, usize), Error> {
    let height = elems.len();
    let width = elems[0].len();

    for row in &elems[1..] {
        if row.len() != width {
            return Err("inconsistent mo_coefs shape".into());
        }
    }

    Ok((height, width))
}

pub fn from_arg(arg: &str) -> Result<Array2<f64>, Error> {
    let elems = arg.split_terminator(&['[', ']'])
                   .filter(|s| s.trim() != "")
                   .map(|s| {
                       s.split(' ')
                        .filter(|t| t.trim() != "")
                        .collect::<Vec<_>>()
                   })
                   .collect::<Vec<_>>();
    let shape = find_shape(&elems)?;
    let mut mo_coefs = Array2::zeros(shape);

    // parse and copy in this scope to propogate errors
    for i in 0..(shape.0) {
        for j in 0..(shape.1) {
            mo_coefs[(i, j)] = elems[i][j].parse()?;
        }
    }

    Ok(mo_coefs)
}
