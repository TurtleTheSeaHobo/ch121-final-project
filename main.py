#!/usr/bin/env python3
import argparse as ap
from pyscf import scf, gto

def struct(**kwargs):
    return type("", (object,), kwargs)()

def parse_ctab(text):
    lines = text.splitlines()
    title = lines[0]
    program = lines[1][2:10]
    timestamp = lines[1][10:]
    comment = lines[2]
    counts = lines[3].split()
    num_atoms = int(counts[0])

    def parse_atom(atom_str):
        fields = atom_str.split()
        return struct(pos = fields[0:3],
                      elem = fields[3])

    atoms_begin = 4
    atoms_end = atoms_begin + num_atoms
    atoms = [parse_atom(ln) for ln in lines[atoms_begin:atoms_end]]

    return struct(title = title,
                  program = program,
                  timestamp = timestamp,
                  comment = comment,
                  atoms = atoms)

def collect_atoms(mol):
    atoms = []

    for i in range(mol.natm):
        coords = list(mol.atom_coord(i))
        atom = (i, coords)
        atoms.append(atom)

    return atoms

def collect_basis_set(mol):
    basis_set = { }

    for i in range(mol.nbas):
        atom = mol.bas_atom(i);
        l = mol.bas_angular(i);

        if atom not in basis_set.keys():
            basis_set[atom] = []

        if l == 0:
            expns = list(mol.bas_exp(i))
            basis_set[atom].append((expns, []))

        coefs = [c[0] for c in mol.bas_ctr_coeff(i)]
        basis_set[atom][-1][1].append(coefs)

    return basis_set

def main():
    parser = ap.ArgumentParser(prog = "ch121-final",
                               description = "calculate and visualize molecular orbitals.")
    parser.add_argument("path",
                        help = "path to ctab (.mol) file")
    parser.add_argument("-b", "--basis",
                        help = "basis set to use",
                        default = "sto-3g")

    args = parser.parse_args()
    ctab = None

    with open(args.path, "r") as file:
        ctab = parse_ctab(file.read())

    mol_str = "; ".join([" ".join([a.elem] + a.pos) for a in ctab.atoms])
    mol = gto.M(atom = mol_str, basis = args.basis)
    mf = scf.hf.SCF(mol)
    mf.scf()

    atoms = collect_atoms(mol)
    basis_set = collect_basis_set(mol)
    mo_coefs = mf.mo_coeff

    print(atoms)
    print(basis_set)
    print(mo_coefs)

if __name__ == "__main__":
    main()
