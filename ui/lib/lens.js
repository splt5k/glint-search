// Refração do vidro: o fundo desfocado é deformado perto das bordas do vidro
// (filtro SVG feDisplacementMap com um mapa gerado a partir da distância até a borda).
export const PAD = 64; // margem extra do wrapper: precisa cobrir ~3x o blur para não escurecer as bordas

// Parâmetros dos materiais (cores/véu ficam em style.css). `k` = força da refração em relação à faixa de borda.
// Saturação baixa de propósito: o fundo do desktop não pode virar neon.
const MATERIALS = {
  clear: { blur: 9, saturate: 1.05, k: 1.3 },
  regular: { blur: 20, saturate: 0.95, k: 1.1 },
};
const NS = 'http://www.w3.org/2000/svg';

let defs;
function ensureDefs() {
  if (defs) return defs;
  const svg = document.createElementNS(NS, 'svg');
  svg.setAttribute('width', '0');
  svg.setAttribute('height', '0');
  svg.style.cssText = 'position:absolute;pointer-events:none';
  defs = document.createElementNS(NS, 'defs');
  svg.appendChild(defs);
  document.body.appendChild(svg);
  return defs;
}

// Mapa de deslocamento: R,G = vetor (para dentro) que cresce perto da borda; B fixo.
function makeMap(w, h, r, band) {
  const k = 0.5; // resolução do mapa (metade basta; o filtro estica)
  const cw = Math.max(2, Math.round((w + PAD * 2) * k));
  const ch = Math.max(2, Math.round((h + PAD * 2) * k));
  const c = document.createElement('canvas');
  c.width = cw; c.height = ch;
  const ctx = c.getContext('2d');
  const img = ctx.createImageData(cw, ch);
  const d = img.data;
  const cx = (w + PAD * 2) / 2, cy = (h + PAD * 2) / 2;
  const hx = w / 2 - r, hy = h / 2 - r;

  for (let y = 0; y < ch; y++) {
    for (let x = 0; x < cw; x++) {
      const px = x / k - cx, py = y / k - cy;
      const qx = Math.abs(px) - hx, qy = Math.abs(py) - hy;
      const ox = Math.max(qx, 0), oy = Math.max(qy, 0);
      const out = Math.hypot(ox, oy) + Math.min(Math.max(qx, qy), 0) - r; // <0 dentro
      const depth = -out; // distância até a borda (positiva dentro)
      let vx = 0, vy = 0;
      if (depth > 0 && depth < band) {
        const t = Math.pow(1 - depth / band, 2.2);
        // normal para fora
        let nx, ny;
        if (qx > 0 || qy > 0) {
          const len = Math.hypot(ox, oy) || 1;
          nx = (ox / len) * Math.sign(px); ny = (oy / len) * Math.sign(py);
        } else if (qx > qy) { nx = Math.sign(px); ny = 0; } else { nx = 0; ny = Math.sign(py); }
        vx = -nx * t; vy = -ny * t; // para dentro
      }
      const i = (y * cw + x) * 4;
      d[i] = 128 + 127 * vx;
      d[i + 1] = 128 + 127 * vy;
      d[i + 2] = 128;
      d[i + 3] = 255;
    }
  }
  ctx.putImageData(img, 0, 0);
  return c.toDataURL('image/png');
}

let counter = 0;

/** Envolve o .glass-bg num wrapper com filtro de refração e mantém o mapa sincronizado com o tamanho. */
export function attachLens(glass) {
  const { blur, saturate, k } = glass.classList.contains('clear') ? MATERIALS.clear : MATERIALS.regular;
  const bg = glass.querySelector(':scope > .glass-bg');
  if (!bg || glass.dataset.lens) return;
  const id = `lens${++counter}`;
  glass.dataset.lens = id;

  const lens = document.createElement('div');
  lens.className = 'glass-lens';
  lens.style.inset = `${-PAD}px`;
  glass.insertBefore(lens, bg);
  lens.appendChild(bg);

  const f = document.createElementNS(NS, 'filter');
  f.setAttribute('id', id);
  f.setAttribute('x', '0'); f.setAttribute('y', '0'); f.setAttribute('width', '100%'); f.setAttribute('height', '100%');
  f.setAttribute('color-interpolation-filters', 'sRGB');
  f.innerHTML = `
    <feGaussianBlur in="SourceGraphic" stdDeviation="${blur}" result="b"/>
    <feColorMatrix in="b" type="saturate" values="${saturate}" result="s"/>
    <feImage result="m" x="0" y="0" width="1" height="1" preserveAspectRatio="none"/>
    <feDisplacementMap in="s" in2="m" scale="0" xChannelSelector="R" yChannelSelector="G"/>`;
  ensureDefs().appendChild(f);
  lens.style.filter = `url(#${id}) var(--m-post)`;

  const img = f.querySelector('feImage');
  let timer = 0, last = '';
  const update = () => {
    const w = glass.offsetWidth, h = glass.offsetHeight;
    if (!w || !h || `${w}x${h}` === last) return;
    last = `${w}x${h}`;
    const r = parseFloat(getComputedStyle(glass).borderTopLeftRadius) || 26;
    const band = Math.min(34, Math.max(9, Math.min(w, h) * 0.2)); // faixa estreita: em peças pequenas a refração não vira um disco interno
    f.querySelector('feDisplacementMap').setAttribute('scale', Math.round(band * k));
    const url = makeMap(w, h, r, band);
    img.setAttribute('href', url);
    img.setAttribute('width', w + PAD * 2);
    img.setAttribute('height', h + PAD * 2);
  };
  new ResizeObserver(() => { clearTimeout(timer); timer = setTimeout(update, 40); }).observe(glass);
  update();
}
