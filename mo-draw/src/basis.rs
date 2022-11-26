use ndarray::{ Array2 };
use crate::error::Error;

pub fn subshell_index(n: usize, l: usize) -> usize {
    let offset = [0, 1, 3, 6, 10][n - 1];

    offset + l
}

#[derive(Debug)]
pub struct Basis {
    pub order:  usize,
    pub expns:  Array2<f64>,
    pub coefs:  Array2<f64>,
}

impl Basis {
    // there are so many better ways to do this that I didn't have time for
    pub fn from_argv(argv: Vec<String>) -> Result<Self, Error> {
        let mut coefs: Vec<Vec<f64>> = Vec::new();
        let mut expns: Vec<Vec<f64>> = Vec::new();

        enum State {
            None,
            PushExpn(usize),
            PushCoef(usize, usize),
        }

        let mut state = State::None;

        for item in argv {
            if &item[..1] == "N" {
                let n: usize = item[1..].parse()?;

                if n < 1 {
                    return Err("invalid N in basis specfication".into());
                } else if n > expns.len() {
                    expns.resize(n, Vec::new());
                }

                state = State::PushExpn(n);
            } else if &item[..1] == "L" {
                let l: usize = item[1..].parse()?;
                let n = match state {
                    State::None => return Err("invalid basis specficiation".into()),
                    State::PushExpn(n) => n,
                    State::PushCoef(n, _l) => n,
                };
                let i = subshell_index(n, l);

                if i >= coefs.len() {
                    coefs.resize(i + 1, Vec::new());
                }

                state = State::PushCoef(n, l);
            } else if let State::PushExpn(n) = state {
                let expn: f64 = item.parse()?;
                let i = n - 1;

                expns[i].push(expn);
            } else if let State::PushCoef(n, l) = state {
                let coef: f64 = item.parse()?;
                let i = subshell_index(n, l);

                coefs[i].push(coef);
            } else {
                return Err("invalid basis specficiation".into());
            }
        }

        fn get_order(expns: &Vec<Vec<f64>>,
                     coefs: &Vec<Vec<f64>>) -> Result<usize, Error> {
            let order = expns[0].len();
            let vecs = expns[1..].into_iter()
                                 .chain(coefs.into_iter());

            for vec in vecs {
                if vec.len() != order {
                    return Err("inconsistent basis order".into());
                }
            }

            Ok(order)
        }

        let order = get_order(&expns, &coefs)?;
        let expns = Array2::from_shape_fn((expns.len(), order),
                                          |(i, j)| expns[i][j]);
        let coefs = Array2::from_shape_fn((coefs.len(), order),
                                          |(i, j)| coefs[i][j]);

        Ok(Self { order, expns, coefs })
    }
}
