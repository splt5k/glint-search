// Conversão de unidades e moedas: "10 km em mi", "30 c para f", "100 usd em brl".
const U = {};
const add = (group, base, ...names) => names.forEach((n) => (U[n.toLowerCase()] = { group, base }));

add('len', 1, 'm', 'metro', 'metros', 'meter', 'meters');
add('len', 1000, 'km', 'quilometro', 'quilometros', 'quilômetro', 'quilômetros');
add('len', 0.01, 'cm', 'centimetro', 'centimetros', 'centímetro', 'centímetros');
add('len', 0.001, 'mm', 'milimetro', 'milimetros', 'milímetro', 'milímetros');
add('len', 1609.344, 'mi', 'milha', 'milhas', 'mile', 'miles');
add('len', 0.9144, 'yd', 'jarda', 'jardas');
add('len', 0.3048, 'ft', 'pe', 'pé', 'pes', 'pés', 'feet', 'foot');
add('len', 0.0254, 'in', 'pol', 'polegada', 'polegadas', 'inch');
add('mass', 1, 'g', 'grama', 'gramas');
add('mass', 1000, 'kg', 'quilo', 'quilos', 'quilograma', 'quilogramas');
add('mass', 0.001, 'mg', 'miligrama', 'miligramas');
add('mass', 1e6, 't', 'ton', 'tonelada', 'toneladas');
add('mass', 453.59237, 'lb', 'lbs', 'libra', 'libras', 'pound', 'pounds');
add('mass', 28.349523125, 'oz', 'onça', 'onca', 'onças');
add('vol', 1, 'l', 'litro', 'litros');
add('vol', 0.001, 'ml', 'mililitro', 'mililitros');
add('vol', 3.785411784, 'gal', 'galão', 'galao', 'galões');
add('vol', 0.2365882365, 'cup', 'xicara', 'xícara', 'xicaras', 'xícaras');
add('time', 1, 's', 'seg', 'segundo', 'segundos');
add('time', 60, 'min', 'minuto', 'minutos');
add('time', 3600, 'h', 'hr', 'hora', 'horas');
add('time', 86400, 'd', 'dia', 'dias');
add('time', 604800, 'sem', 'semana', 'semanas');
add('data', 1, 'b', 'byte', 'bytes');
add('data', 1024, 'kb', 'kib');
add('data', 1024 ** 2, 'mb', 'mib');
add('data', 1024 ** 3, 'gb', 'gib');
add('data', 1024 ** 4, 'tb', 'tib');
add('speed', 1, 'm/s');
add('speed', 1 / 3.6, 'km/h', 'kmh', 'kph');
add('speed', 0.44704, 'mph');
['c', '°c', 'celsius'].forEach((n) => (U[n] = { group: 'tc' }));
['f', '°f', 'fahrenheit'].forEach((n) => (U[n] = { group: 'tf' }));
['k', 'kelvin'].forEach((n) => (U[n] = { group: 'tk' }));

const RE = /^\s*(-?\d+(?:[.,]\d+)?)\s*([^\d\s][^\s]*?)\s+(?:to|em|in|para|->|=|até)\s+([^\d\s][^\s]*)\s*$/i;

const fmt = (v) =>
  new Intl.NumberFormat('pt-BR', { maximumFractionDigits: Math.abs(v) >= 1000 || Number.isInteger(v) ? 2 : 4 }).format(v);

// ---- moedas: taxas baixadas uma vez e guardadas por 6h ----
let rates = null;
try {
  const c = JSON.parse(localStorage.getItem('rates') || 'null');
  if (c && Date.now() - c.at < 6 * 3600e3) rates = c.rates;
} catch {}

export async function warmRates() {
  if (rates) return;
  try {
    const r = await fetch('https://open.er-api.com/v6/latest/USD');
    const j = await r.json();
    rates = j.rates;
    localStorage.setItem('rates', JSON.stringify({ at: Date.now(), rates }));
  } catch {}
}

export function convert(input) {
  const m = input.match(RE);
  if (!m) return null;
  const value = parseFloat(m[1].replace(',', '.'));
  const from = m[2].toLowerCase();
  const to = m[3].toLowerCase();

  if (rates && from.length === 3 && to.length === 3 && rates[from.toUpperCase()] && rates[to.toUpperCase()]) {
    const r = (value / rates[from.toUpperCase()]) * rates[to.toUpperCase()];
    return { title: `${fmt(r)} ${to.toUpperCase()}`, sub: `${fmt(value)} ${from.toUpperCase()} · cotação atual`, copy: fmt(r) };
  }

  const a = U[from], b = U[to];
  if (!a || !b) return null;
  let r;
  if (a.group.startsWith('t') && a.group.length === 2 && b.group.length === 2 && b.group.startsWith('t')) {
    const c = a.group === 'tc' ? value : a.group === 'tf' ? ((value - 32) * 5) / 9 : value - 273.15;
    r = b.group === 'tc' ? c : b.group === 'tf' ? (c * 9) / 5 + 32 : c + 273.15;
  } else if (a.group === b.group && a.base) {
    r = (value * a.base) / b.base;
  } else return null;
  return { title: `${fmt(r)} ${m[3]}`, sub: `${fmt(value)} ${m[2]} → ${m[3]}`, copy: fmt(r) };
}
