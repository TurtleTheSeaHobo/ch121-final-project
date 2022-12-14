use ndarray::{ Array2 };
use crate::error::Error;

pub fn nl_index(n: i32, l: i32) -> usize {
    const OFFSETS: [usize; 5] = [0, 1, 3, 6, 10];
    OFFSETS[n as usize - 1] + l as usize
}

pub fn lm_index(l: i32, m: i32) -> usize {
    (2 * l + m) as usize
}

#[derive(Debug)]
pub struct Basis {
    pub order:  usize,
    pub expns:  Array2<f64>,
    pub coefs:  Array2<f64>,
}

impl Basis {
    // there are so many better ways to do this that I didn't have time for
    pub fn from_arg(arg: &str) -> Result<Self, Error> {
        let mut coefs: Vec<Vec<f64>> = Vec::new();
        let mut expns: Vec<Vec<f64>> = Vec::new();

        enum State {
            None,
            PushExpn(i32),
            PushCoef(i32, i32),
        }

        let mut state = State::None;

        for item in arg.trim_matches('[')
                       .trim_matches(']')
                       .split(' ') {
            if &item[..1] == "N" {
                let n: i32 = item[1..].parse()?;

                if n < 1 {
                    return Err("invalid N in basis specfication".into());
                } else if (n as usize) > expns.len() {
                    expns.resize(n as usize, Vec::new());
                }

                state = State::PushExpn(n);
            } else if &item[..1] == "L" {
                let l: i32 = item[1..].parse()?;
                let n = match state {
                    State::None => return Err("invalid basis specficiation".into()),
                    State::PushExpn(n) => n,
                    State::PushCoef(n, _l) => n,
                };
                let i = nl_index(n, l);

                if i >= coefs.len() {
                    coefs.resize(i + 1, Vec::new());
                }

                state = State::PushCoef(n, l);
            } else if let State::PushExpn(n) = state {
                let expn: f64 = item.parse()?;
                let i = (n - 1) as usize;

                expns[i].push(expn);
            } else if let State::PushCoef(n, l) = state {
                let coef: f64 = item.parse()?;
                let i = nl_index(n, l);

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
