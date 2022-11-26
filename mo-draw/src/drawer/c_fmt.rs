use ndarray::{ Array1, ArrayBase, Ix1, Ix2, RawData, Data };
use crate::{
    atom::Atom,
    basis::{ self, Basis },
};

pub fn array1<S, T>(array: &ArrayBase<S, Ix1>) -> String
where
    S: RawData<Elem = T> + Data,
    T: ToString,
{
    let s = array.iter()
                 .map(|x| x.to_string())
                 .collect::<Vec<String>>()
                 .join(", ");

    format!("{{ {s} }}")
}

pub fn array2<S, T>(array: &ArrayBase<S, Ix2>) -> String
where
    S: RawData<Elem = T> + Data,
    T: ToString,
{
    let s = array.rows()
                 .into_iter()
                 .map(|r| array1(&r))
                 .collect::<Vec<String>>()
                 .join(", ");

    format!("{{ {s} }}")
}

pub fn orbitals(atoms: &Array1<Atom>,
                bases: &Array1<Basis>) -> String {
    let mut v = Vec::new();

    for atom in atoms {
        let basis = &bases[atom.basis_id];
        let pos = format!("{{ {x}, {y}, {z} }}",
                          x = atom.position[0],
                          y = atom.position[1],
                          z = atom.position[2]);

        for n in 1..=basis.expns.len() {
            let bas_expns = array1(&basis.expns.row(n - 1));

            for l in 0..n {
                let nl_idx = basis::subshell_index(n, l);
                let bas_coefs = array1(&basis.coefs.row(nl_idx));
                let entry = format!("{{ {pos}, {nl_idx}, {bas_expns}, {bas_coefs} }}");

                v.push(entry);
            }
        }
    }

    format!("{{ {s} }}", s = v.join(", "))
}

