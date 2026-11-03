// Interpretador de datas/horas em português: "amanhã 14h", "sexta 18:30", "15/11 9h", "hoje meio-dia".
const DAYS = ['domingo', 'segunda', 'terça', 'quarta', 'quinta', 'sexta', 'sábado'];
const MONTHS = ['janeiro', 'fevereiro', 'março', 'abril', 'maio', 'junho', 'julho', 'agosto', 'setembro', 'outubro', 'novembro', 'dezembro'];

const norm = (s) => s.toLowerCase().normalize('NFD').replace(/[̀-ͯ]/g, '');

export function parseWhen(text, base = new Date()) {
  const s = norm(text.trim());
  if (!s) return null;
  const d = new Date(base);
  d.setSeconds(0, 0);
  let hasDate = false, hasTime = false;

  // hora
  let m;
  if (/meio[- ]?dia/.test(s)) { d.setHours(12, 0); hasTime = true; }
  else if (/meia[- ]?noite/.test(s)) { d.setHours(23, 59); hasTime = true; }
  else if ((m = s.match(/(\d{1,2})\s*[:h]\s*(\d{2})?(?!\d)/))) {
    d.setHours(Math.min(+m[1], 23), m[2] ? Math.min(+m[2], 59) : 0); hasTime = true;
  } else if ((m = s.match(/\bas?\s+(\d{1,2})\b(?!\/)/))) {
    d.setHours(Math.min(+m[1], 23), 0); hasTime = true;
  }
  if (/\bda (noite|tarde)\b/.test(s) && d.getHours() < 12) d.setHours(d.getHours() + 12);

  // data
  if (/depois de amanha/.test(s)) { d.setDate(d.getDate() + 2); hasDate = true; }
  else if (/amanha/.test(s)) { d.setDate(d.getDate() + 1); hasDate = true; }
  else if (/\bhoje\b/.test(s)) hasDate = true;
  else if ((m = s.match(/\b(\d{1,2})\/(\d{1,2})(?:\/(\d{2,4}))?/))) {
    d.setMonth(+m[2] - 1, +m[1]);
    if (m[3]) d.setFullYear(m[3].length === 2 ? 2000 + +m[3] : +m[3]);
    hasDate = true;
  } else if ((m = s.match(new RegExp(`\\b(\\d{1,2}) de (${MONTHS.map(norm).join('|')})`)))) {
    d.setMonth(MONTHS.map(norm).indexOf(m[2]), +m[1]);
    hasDate = true;
  } else {
    const di = DAYS.map(norm).findIndex((n) => new RegExp(`\\b${n}`).test(s));
    if (di >= 0) {
      let diff = (di - d.getDay() + 7) % 7;
      if (diff === 0) diff = 7;
      d.setDate(d.getDate() + diff);
      hasDate = true;
    }
  }

  if (!hasDate && !hasTime) return null;
  if (hasDate && !hasTime) d.setHours(9, 0);
  // só hora, já passou hoje → amanhã
  if (!hasDate && hasTime && d < base) d.setDate(d.getDate() + 1);
  return d;
}

const p2 = (n) => String(n).padStart(2, '0');
export const toIcs = (d) => `${d.getFullYear()}${p2(d.getMonth() + 1)}${p2(d.getDate())}T${p2(d.getHours())}${p2(d.getMinutes())}00`;

export function describe(d) {
  const today = new Date();
  const same = (a, b) => a.toDateString() === b.toDateString();
  const tomorrow = new Date(today); tomorrow.setDate(today.getDate() + 1);
  const day = same(d, today) ? 'hoje' : same(d, tomorrow) ? 'amanhã' : d.toLocaleDateString('pt-BR', { weekday: 'short', day: '2-digit', month: 'short' });
  return `${day} às ${p2(d.getHours())}:${p2(d.getMinutes())}`;
}

// "10 min", "1h30", "45s", "2 horas" → segundos
export function parseDuration(text) {
  const s = norm(text);
  let total = 0, found = false;
  const re = /(\d+(?:[.,]\d+)?)\s*(h|hr|hora|horas|min|m|minuto|minutos|s|seg|segundo|segundos)\b/g;
  let m;
  while ((m = re.exec(s))) {
    const n = parseFloat(m[1].replace(',', '.'));
    const u = m[2];
    total += /^h/.test(u) ? n * 3600 : /^(min|m$|minuto)/.test(u) ? n * 60 : n;
    found = true;
  }
  if (!found && /^\d+$/.test(s.trim())) { total = +s.trim() * 60; found = true; }
  return found ? total : null;
}
