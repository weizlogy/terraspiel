/**
 * Varies a hex color by a random amount.
 * @param hex The base hex color string (e.g., '#8B4513').
 * @param amount The maximum percentage to vary the lightness (e.g., 15 for +/- 15%).
 * @returns A new hex color string.
 */
export const varyColor = (hex: string, amount: number = 15): string => {
  // Convert hex to HSL
  let r = parseInt(hex.slice(1, 3), 16) / 255;
  let g = parseInt(hex.slice(3, 5), 16) / 255;
  let b = parseInt(hex.slice(5, 7), 16) / 255;

  let max = Math.max(r, g, b), min = Math.min(r, g, b);
  let h = 0, s = 0, l = (max + min) / 2;

  if (max !== min) {
    let d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    switch (max) {
      case r: h = (g - b) / d + (g < b ? 6 : 0); break;
      case g: h = (b - r) / d + 2; break;
      case b: h = (r - g) / d + 4; break;
    }
    h /= 6;
  }

  // Vary lightness
  const factor = (Math.random() - 0.5) * 2 * (amount / 100);
  l = Math.max(0, Math.min(1, l + l * factor));

  // Convert HSL back to RGB
  let r_new: number, g_new: number, b_new: number;
  if (s === 0) {
    r_new = g_new = b_new = l; // achromatic
  } else {
    const hue2rgb = (p: number, q: number, t: number) => {
      if (t < 0) t += 1;
      if (t > 1) t -= 1;
      if (t < 1/6) return p + (q - p) * 6 * t;
      if (t < 1/2) return q;
      if (t < 2/3) return p + (q - p) * (2/3 - t) * 6;
      return p;
    };
    let q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    let p = 2 * l - q;
    r_new = hue2rgb(p, q, h + 1/3);
    g_new = hue2rgb(p, q, h);
    b_new = hue2rgb(p, q, h - 1/3);
  }

  // Convert RGB to Hex
  const toHex = (c: number) => {
    const hex = Math.round(c * 255).toString(16);
    return hex.length === 1 ? '0' + hex : hex;
  };

  return `#${toHex(r_new)}${toHex(g_new)}${toHex(b_new)}`;
};