use crate::error::Error;

#[derive(Debug)]
pub struct Atom {
    pub basis_id:   usize,
    pub position:   [f64; 3],
}

impl Atom {
    pub fn from_arg(arg: &str) -> Result<Self, Error> {
        let mut basis_id = 0;
        let mut position = [0.0; 3];

        for item in arg.trim_matches('[')
                       .trim_matches(']')
                       .split(' ') {
            match (&item[..1], &item[1..]) {
                ("B", rest) => basis_id = rest.parse()?,
                ("X", rest) => position[0] = rest.parse()?,
                ("Y", rest) => position[1] = rest.parse()?,
                ("Z", rest) => position[2] = rest.parse()?,
                _ => return Err("invalid atom specification".into()),
            }
        }

        Ok(Self { basis_id, position })
    }
}
