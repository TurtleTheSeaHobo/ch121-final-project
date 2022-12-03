#version 460

const float pi = 3.1415927;
const float step_len = @STEP_LEN;
const int num_steps = @NUM_STEPS;
const int num_cc = @NUM_CC;

struct Orbital {
    vec3            org;
    int             lm_idx;
    float[num_cc]   bas_expns;
    float[num_cc]   bas_coefs;
};

const Orbital[] orbitals = @ORBITALS;
const int num_ao = orbitals.length();
const int num_mo = num_ao; // feels more expressive
const float[num_mo][num_ao] mo_coefs = @MO_COEFS;

uniform vec2 resolution;
uniform int mo_idx;

out vec4 f_color;

// works for l on [0, 2]
// also, lm_idx must be in lock-step or this will be really slow
float re_sph(vec3 pos, int lm_idx) {
    float r = pos.length();

    switch (lm_idx) {
    case 0:
        return sqrt(0.25    / pi);
    case 1:
        return sqrt(0.75    / pi) * (pos.y / r);
    case 2:
        return sqrt(0.75    / pi) * (pos.z / r);
    case 3:
        return sqrt(0.75    / pi) * (pos.x / r);
    case 4:
        return sqrt(0.375   / pi) * (pos.x * pos.y / (r * r));
    case 5:
        return sqrt(0.375   / pi) * (pos.y * pos.z / (r * r));
    case 6:
        return sqrt(0.3125  / pi) * (3.0 * pos.z * pos.z / (r * r) - 1.0);
    case 7:
        return sqrt(0.375   / pi) * (pos.x * pos.z / (r * r));
    case 8:
        return sqrt(0.9375  / pi) * ((pos.x * pos.x) - (pos.y * pos.y)) / (r * r);
    }
}

float wave_fn(vec3 pos, int ao_idx) {
    pos -= orbitals[ao_idx].org;
    float r2 = dot(pos, pos);
    float radial = 0.0;

    for (int i = 0; i < num_cc; i++) {
        float coef = orbitals[ao_idx].bas_coefs[i];
        float expn = orbitals[ao_idx].bas_expns[i];
        radial += coef * pow(2.0 * expn / pi, 0.75) * exp(-expn * r2);
    }


    return radial * re_sph(pos, orbitals[ao_idx].lm_idx);
}

float lcao_wave_fn(vec3 pos) {
    float w = 0.0;

    for (int i = 0; i < num_ao; i++) {
        w += wave_fn(pos, i) * mo_coefs[mo_idx][i];
    }

    return w;
}

float ray_integral(vec3 ro, vec3 rd) {
    float rm = 5.0;
    float q = 0.0;
    float wa = lcao_wave_fn(ro);
    float wb = 0.0;

    for (int i = 0; i < num_steps; i++) {
        vec3 pos = ro + rd * rm;

        wb = lcao_wave_fn(pos);
        q += 0.5 * step_len * (wa + wb);

        wa = wb;
        rm += step_len;
    }

    return q * q;
}

void main() {
    vec2 uv = (gl_FragCoord.xy - 0.5 * resolution) / resolution.y;
    vec3 ro = vec3(0.0, 0.0, -10.0);
    vec3 rd = normalize(vec3(uv, 1.0));
    float q = ray_integral(ro, rd);

    f_color = vec4(vec3(q) * 10.0, 1.0);
}
