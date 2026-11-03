// Calculadora segura (sem eval): + - * / ^ % ( ) , funções e "15% de 200".
const FUNCS = {
  sqrt: Math.sqrt, raiz: Math.sqrt, abs: Math.abs, round: Math.round, floor: Math.floor, ceil: Math.ceil,
  sin: (x) => Math.sin((x * Math.PI) / 180), cos: (x) => Math.cos((x * Math.PI) / 180),
  tan: (x) => Math.tan((x * Math.PI) / 180), log: Math.log10, ln: Math.log, exp: Math.exp,
};
const CONSTS = { pi: Math.PI, e: Math.E };

function tokenize(src) {
  const out = [];
  let i = 0;
  while (i < src.length) {
    const c = src[i];
    if (/\s/.test(c)) { i++; continue; }
    if (/[0-9.,]/.test(c)) {
      let j = i;
      while (j < src.length && /[0-9.,]/.test(src[j])) j++;
      let raw = src.slice(i, j);
      // "1.234,56" (pt-BR) ou "1,234.56" (en) ou "3,5"
      if (raw.includes(',') && raw.includes('.')) {
        raw = raw.lastIndexOf(',') > raw.lastIndexOf('.') ? raw.replace(/\./g, '').replace(',', '.') : raw.replace(/,/g, '');
      } else if (raw.includes(',')) raw = raw.replace(',', '.');
      const n = Number(raw);
      if (Number.isNaN(n)) return null;
      out.push({ t: 'num', v: n });
      i = j;
    } else if (/[a-zA-Zπ√]/.test(c)) {
      let j = i;
      while (j < src.length && /[a-zA-Zπ√]/.test(src[j])) j++;
      let w = src.slice(i, j).toLowerCase();
      if (w === 'π') w = 'pi';
      if (w === '√') w = 'sqrt';
      out.push({ t: 'id', v: w });
      i = j;
    } else if ('+-*/^%()x×÷'.includes(c)) {
      out.push({ t: 'op', v: c === 'x' || c === '×' ? '*' : c === '÷' ? '/' : c });
      i++;
    } else return null;
  }
  return out;
}

function parse(tokens) {
  let p = 0;
  const peek = () => tokens[p];
  const next = () => tokens[p++];

  function primary() {
    const t = next();
    if (!t) throw 0;
    if (t.t === 'num') {
      let v = t.v;
      if (peek() && peek().v === '%' && !(tokens[p + 1] && (tokens[p + 1].t === 'num' || tokens[p + 1].v === '('))) { next(); v /= 100; }
      return v;
    }
    if (t.t === 'id') {
      if (t.v in CONSTS) return CONSTS[t.v];
      if (t.v in FUNCS) {
        if (!peek() || peek().v !== '(') throw 0;
        next();
        const a = expr();
        if (!peek() || next().v !== ')') throw 0;
        return FUNCS[t.v](a);
      }
      throw 0;
    }
    if (t.v === '(') {
      const v = expr();
      if (!peek() || next().v !== ')') throw 0;
      return v;
    }
    if (t.v === '-') return -power();
    if (t.v === '+') return power();
    throw 0;
  }
  function power() {
    let b = primary();
    if (peek() && peek().v === '^') { next(); b = Math.pow(b, power()); }
    return b;
  }
  function term() {
    let v = power();
    while (peek() && '*/%'.includes(peek().v) && peek().t === 'op') {
      const op = next().v;
      const r = power();
      v = op === '*' ? v * r : op === '/' ? v / r : v % r;
    }
    return v;
  }
  function expr() {
    let v = term();
    while (peek() && (peek().v === '+' || peek().v === '-') && peek().t === 'op') {
      const op = next().v;
      const r = term();
      v = op === '+' ? v + r : v - r;
    }
    return v;
  }
  const v = expr();
  if (p !== tokens.length) throw 0;
  return v;
}

export function fmt(n) {
  if (!Number.isFinite(n)) return null;
  const abs = Math.abs(n);
  if (abs !== 0 && (abs >= 1e15 || abs < 1e-9)) return n.toExponential(6);
  return new Intl.NumberFormat('pt-BR', { maximumFractionDigits: 10 }).format(n);
}

export function calculate(input) {
  let s = input.trim();
  if (!/[0-9πe]/.test(s)) return null;

  // "15% de 200" / "15% of 200"
  const m = s.match(/^(-?[\d.,]+)\s*%\s*(?:de|of)\s*(-?[\d.,]+)$/i);
  if (m) {
    const a = parse(tokenize(m[1]) || []);
    const b = parse(tokenize(m[2]) || []);
    return fmt((a / 100) * b);
  }

  // precisa parecer conta: ao menos um operador/função, ou "sqrt(...)"
  if (!/[+\-*/^%×÷x()]|sqrt|raiz|sin|cos|tan|log|ln|exp|abs|round|floor|ceil|π|pi/i.test(s)) return null;
  if (/^[\d.,\s]+$/.test(s)) return null;
  const toks = tokenize(s);
  if (!toks || !toks.length) return null;
  try {
    return fmt(parse(toks));
  } catch {
    return null;
  }
}
