#!/usr/bin/env python3
import argparse as ap
import subprocess as sp
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

def run_mo_draw(basis_set, atoms, mo_coefs):
    def to_b_arg(basis):
        n = 1
        s = "["

        for (expns, coefses) in basis:
            s += "N{n} ".format(n = n)

            for expn in expns:
                s += "{x} ".format(x = expn)

            l = 0

            for coefs in coefses:
                s += "L{l} ".format(l = l)

                for coef in coefs:
                    s += "{x} ".format(x = coef)

                l += 1

            n += 1

        s = s[:-1] + "]"

        return s

    def to_a_arg(atom):
        (b, pos) = atom
        s = "[B{b} X{x} Y{y} Z{z}]".format(b = b,
                                             x = pos[0],
                                             y = pos[1],
                                             z = pos[2])

        return s

    def to_c_arg(mo_coefs):
        s = "["

        for row in mo_coefs:
            s += "["

            for coef in row:
                s += "{x} ".format(x = coef)

            s = s[:-1] + "] "

        s = s[:-1] + "]"

        return s

    b_args = [to_b_arg(basis) for basis in basis_set.values()]
    a_args = [to_a_arg(atom) for atom in atoms]
    c_arg = to_c_arg(mo_coefs)

    args = ["./mo-draw/mo-draw"]

    for arg in b_args: args += ["-B", arg]
    for arg in a_args: args += ["-A", arg]

    args += ["-C", c_arg]

    print("Num mo coefs: {x}".format( x = len(mo_coefs[1])))
    #print(args)
    sp.run(args)

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

    basis_set = collect_basis_set(mol)
    atoms = collect_atoms(mol)
    mo_coefs = mf.mo_coeff

    #print(basis_set)
    #print(atoms)
    #print(mo_coefs)
    print("MO (occ, energy):")
    print(list(zip(mf.mo_energy, mf.mo_occ)))

    run_mo_draw(basis_set, atoms, mo_coefs)

if __name__ == "__main__":
    main()
