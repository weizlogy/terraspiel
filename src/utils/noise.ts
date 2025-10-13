
// This code is a port of the public domain Java implementation by Stefan Gustavson.
// The original code can be found here: http://webstaff.itn.liu.se/~stegu/simplexnoise/SimplexNoise.java
// It has been modified for use in this project.

const F2 = 0.5 * (Math.sqrt(3.0) - 1.0);
const G2 = (3.0 - Math.sqrt(3.0)) / 6.0;

const grad3 = new Float32Array([
  1, 1, 0, -1, 1, 0, 1, -1, 0, -1, -1, 0,
  1, 0, 1, -1, 0, 1, 1, 0, -1, -1, 0, -1,
  0, 1, 1, 0, -1, 1, 0, 1, -1, 0, -1, -1,
]);

function createPermutationTable(random: () => number): Uint8Array {
  const p = new Uint8Array(256);
  for (let i = 0; i < 256; i++) {
    p[i] = i;
  }
  for (let i = 255; i > 0; i--) {
    const n = Math.floor((i + 1) * random());
    const q = p[i];
    p[i] = p[n];
    p[n] = q;
  }
  return p;
}

export function createNoise2D(random: () => number = Math.random) {
  const p = createPermutationTable(random);
  const perm = new Uint8Array(512);
  const permMod12 = new Uint8Array(512);
  for (let i = 0; i < 512; i++) {
    perm[i] = p[i & 255];
    permMod12[i] = perm[i] % 12;
  }

  return (x: number, y: number): number => {
    let n0, n1, n2; // Noise contributions from the three corners

    const s = (x + y) * F2;
    const i = Math.floor(x + s);
    const j = Math.floor(y + s);
    const t = (i + j) * G2;
    const X0 = i - t;
    const Y0 = j - t;
    const x0 = x - X0;
    const y0 = y - Y0;

    let i1, j1; // Offsets for second corner of simplex in (i,j) coords
    if (x0 > y0) {
      i1 = 1;
      j1 = 0;
    } else {
      i1 = 0;
      j1 = 1;
    }

    const x1 = x0 - i1 + G2;
    const y1 = y0 - j1 + G2;
    const x2 = x0 - 1.0 + 2.0 * G2;
    const y2 = y0 - 1.0 + 2.0 * G2;

    const ii = i & 255;
    const jj = j & 255;

    let t0 = 0.5 - x0 * x0 - y0 * y0;
    if (t0 < 0) n0 = 0.0;
    else {
      t0 *= t0;
      const g = permMod12[ii + perm[jj]];
      n0 = t0 * t0 * (grad3[g * 3] * x0 + grad3[g * 3 + 1] * y0);
    }

    let t1 = 0.5 - x1 * x1 - y1 * y1;
    if (t1 < 0) n1 = 0.0;
    else {
      t1 *= t1;
      const g = permMod12[ii + i1 + perm[jj + j1]];
      n1 = t1 * t1 * (grad3[g * 3] * x1 + grad3[g * 3 + 1] * y1);
    }

    let t2 = 0.5 - x2 * x2 - y2 * y2;
    if (t2 < 0) n2 = 0.0;
    else {
      t2 *= t2;
      const g = permMod12[ii + 1 + perm[jj + 1]];
      n2 = t2 * t2 * (grad3[g * 3] * x2 + grad3[g * 3 + 1] * y2);
    }

    return 70.0 * (n0 + n1 + n2);
  };
}
