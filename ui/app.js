import { calculate } from './lib/calc.js';
import { convert, warmRates } from './lib/units.js';
import { parseWhen, parseDuration, toIcs, describe } from './lib/when.js';
import { attachLens, PAD } from './lib/lens.js';
import { SERVICES } from './lib/services.js';

const T = window.__TAURI__;
const invoke = (c, a) => T.core.invoke(c, a);
const IS_WIN = navigator.userAgent.includes('Windows');
const ICO = IS_WIN ? 'http://ico.localhost/' : 'ico://localhost/';
const $ = (id) => document.getElementById(id);
const enc = encodeURIComponent;
const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));

const input = $('q'), formEl = $('form'), clearBtn = $('clear');
const panel = $('panel'), phead = $('phead'), chipsEl = $('chips'), pbody = $('pbody'), qkbar = $('qkbar'), ql = $('ql');

// ───────────────────────── ícones ─────────────────────────
const svg = (d) => `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${d}</svg>`;
const G = {
  gear: svg('<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 00.3 1.8l.1.1a2 2 0 11-2.8 2.8l-.1-.1a1.7 1.7 0 00-1.8-.3 1.7 1.7 0 00-1 1.5V21a2 2 0 01-4 0v-.1a1.7 1.7 0 00-1.1-1.5 1.7 1.7 0 00-1.8.3l-.1.1a2 2 0 11-2.8-2.8l.1-.1a1.7 1.7 0 00.3-1.8 1.7 1.7 0 00-1.5-1H3a2 2 0 010-4h.1a1.7 1.7 0 001.5-1.1 1.7 1.7 0 00-.3-1.8l-.1-.1a2 2 0 112.8-2.8l.1.1a1.7 1.7 0 001.8.3H9a1.7 1.7 0 001-1.5V3a2 2 0 014 0v.1a1.7 1.7 0 001 1.5 1.7 1.7 0 001.8-.3l.1-.1a2 2 0 112.8 2.8l-.1.1a1.7 1.7 0 00-.3 1.8V9a1.7 1.7 0 001.5 1H21a2 2 0 010 4h-.1a1.7 1.7 0 00-1.5 1z"/>'),
  timer: svg('<circle cx="12" cy="13.5" r="7"/><path d="M12 9.5v4l2.5 1.5M9.5 3h5"/>'),
  note: svg('<path d="M6 3.5h9l4 4V20a1 1 0 01-1 1H6a1 1 0 01-1-1V4.5a1 1 0 011-1zM14.5 3.5V8H19M8.5 12.5h7M8.5 16h5"/>'),
  event: svg('<rect x="4" y="5" width="16" height="15" rx="2.5"/><path d="M4 10h16M8.5 3v4M15.5 3v4"/>'),
  mail: svg('<rect x="3.5" y="5.5" width="17" height="13" rx="2.5"/><path d="m4 7.5 8 6 8-6"/>'),
  msg: svg('<path d="M20 11.5a7.5 7.5 0 01-11 6.6L4 19.5l1.4-4.6A7.5 7.5 0 1120 11.5z"/>'),
  lock: svg('<rect x="5" y="10.5" width="14" height="10" rx="2.5"/><path d="M8 10.5V8a4 4 0 018 0v2.5"/>'),
  moon: svg('<path d="M20 14.5A8 8 0 019.5 4a8 8 0 1010.5 10.5z"/>'),
  power: svg('<path d="M12 3v8M6.8 6.5a7.5 7.5 0 1010.4 0"/>'),
  restart: svg('<path d="M4.5 12a7.5 7.5 0 107-7.5M4.5 4v5h5"/>'),
  trash: svg('<path d="M4.5 7h15M9.5 7V4.5h5V7M6.5 7l1 13h9l1-13"/>'),
  task: svg('<path d="M4 18V9M10 18V5M16 18v-7M21 18H3"/>'),
  term: svg('<rect x="3.5" y="4.5" width="17" height="15" rx="2.5"/><path d="m7.5 10 3 2.5-3 2.5M13 15.5h4"/>'),
  gear: svg('<circle cx="12" cy="12" r="3"/><path d="M12 2.8v2.4M12 18.8v2.4M2.8 12h2.4M18.8 12h2.4M5.5 5.5l1.7 1.7M16.8 16.8l1.7 1.7M5.5 18.5l1.7-1.7M16.8 7.2l1.7-1.7"/>'),
  globe: svg('<circle cx="12" cy="12" r="8.5"/><path d="M3.5 12h17M12 3.5c2.5 2.5 3.5 5.3 3.5 8.5S14.5 18 12 20.5C9.5 18 8.5 15.2 8.5 12S9.5 6 12 3.5z"/>'),
  calc: svg('<rect x="5" y="3.5" width="14" height="17" rx="2.5"/><path d="M8.5 7.5h7M9 12h.01M12 12h.01M15 12h.01M9 16h.01M12 16h.01M15 16h.01"/>'),
  swap: svg('<path d="M4 8h14l-3.5-3.5M20 16H6l3.5 3.5"/>'),
  clip: svg('<rect x="5.5" y="4.5" width="13" height="16.5" rx="2.2"/><path d="M9 4.5V4a1.5 1.5 0 011.5-1.5h3A1.5 1.5 0 0115 4v.5M9 11h6M9 15h4"/>'),
  apps: svg('<path d="M12 3 4.5 20M12 3l7.5 17M7 14.5h10"/>'),
  folder: svg('<path d="M3.5 7.5a2 2 0 012-2h4l2 2.2h7a2 2 0 012 2V17a2 2 0 01-2 2h-13a2 2 0 01-2-2z"/>'),
  bolt: svg('<path d="m12 3 8.5 4.5L12 12 3.5 7.5zM3.5 12 12 16.5l8.5-4.5M3.5 16.5 12 21l8.5-4.5"/>'),
};

// ───────────────────────── catálogo de ações ─────────────────────────
const ACTIONS = [
  { id: 'timer', title: 'Iniciar timer', sub: 'Relógio', icon: 'timer', color: '#ff9f0a', kw: ['timer', 'cronômetro', 'cronometro', 'temporizador', 'alarme'],
    tpl: ['Iniciar timer de', 'dur', 'chamado', 'label'], fields: { dur: '10 minutos', label: 'Nome (opcional)' } },
  { id: 'note', title: 'Criar nota', sub: 'Notas', icon: 'note', color: '#ffd60a', kw: ['nota', 'anotar', 'note', 'lembrar'],
    tpl: ['Criar nota', 'title', ':', 'text'], fields: { title: 'Título', text: 'Texto' } },
  { id: 'event', title: 'Novo evento', sub: 'Calendário', icon: 'event', color: '#ff453a', kw: ['evento', 'reunião', 'reuniao', 'agendar', 'calendário', 'calendario', 'compromisso'],
    tpl: ['Adicionar', 'title', 'de', 'start', 'até', 'end'], fields: { title: 'Título', start: 'amanhã 12h', end: 'amanhã 13h' } },
  { id: 'email', title: 'Enviar e-mail', sub: 'E-mail', icon: 'mail', color: '#0a84ff', kw: ['email', 'e-mail', 'mail', 'enviar email', 'correio'],
    tpl: ['Enviar e-mail para', 'to', 'assunto', 'subject'], fields: { to: 'destinatário', subject: 'Assunto' } },
  { id: 'message', title: 'Enviar mensagem', sub: 'WhatsApp', icon: 'msg', color: '#30d158', kw: ['mensagem', 'whatsapp', 'zap', 'message', 'enviar mensagem'],
    tpl: ['Enviar mensagem', 'text'], fields: { text: 'Mensagem' } },
  { id: 'lock', title: 'Bloquear tela', sub: 'Sistema', icon: 'lock', color: '#636366', kw: ['bloquear', 'lock', 'bloqueio', 'travar'] },
  { id: 'sleep', title: 'Suspender', sub: 'Sistema', icon: 'moon', color: '#5e5ce6', kw: ['suspender', 'sleep', 'dormir', 'hibernar'] },
  { id: 'restart', title: 'Reiniciar computador', sub: 'Sistema', icon: 'restart', color: '#ff9f0a', kw: ['reiniciar', 'restart', 'reboot'], confirm: true },
  { id: 'shutdown', title: 'Desligar computador', sub: 'Sistema', icon: 'power', color: '#ff453a', kw: ['desligar', 'shutdown', 'power off'], confirm: true },
  { id: 'empty_trash', title: 'Esvaziar lixeira', sub: 'Sistema', icon: 'trash', color: '#8e8e93', kw: ['lixeira', 'esvaziar', 'trash', 'recycle'], confirm: true },
  { id: 'taskmgr', title: 'Gerenciador de tarefas', sub: 'Sistema', icon: 'task', color: '#32ade6', kw: ['tarefas', 'taskmgr', 'processos', 'task manager'] },
  { id: 'terminal', title: 'Terminal', sub: 'Sistema', icon: 'term', color: '#1c1c1e', kw: ['terminal', 'cmd', 'powershell', 'prompt', 'console'] },
  { id: 'prefs', title: 'Configurações do Glint', sub: 'Atalho, indexação, tema, exportar/importar', icon: 'gear', color: '#8e8e93', kw: ['configurações', 'configuracoes', 'preferências', 'ajustes', 'settings', 'atalho'], local: true },
];
const A_BY_ID = Object.fromEntries(ACTIONS.map((a) => [a.id, a]));

const GROUP = { service: 'Sites', cmd: 'Ações', app: 'Aplicativos', folder: 'Pastas', file: 'Documentos e arquivos', setting: 'Configurações', action: 'Ações', nlp: 'Ações', calc: 'Calculadora', conv: 'Conversão', web: 'Pesquisar na web', clip: 'Área de transferência' };
const FILE_CHIPS = [['Todos', 'all'], ['Imagens', 'images'], ['Documentos', 'docs'], ['PDFs', 'pdf'], ['Mídia', 'media'], ['Pastas', 'folders']];

// ───────────────────────── estado ─────────────────────────
let filter = null; // null | apps | files | actions | clip
let fileKind = 'all';
let items = [];
let sel = 0;
let cols = 1;
let seq = 0;
let usage = {};
let neverSuggest = new Set(); // ids que o usuário pediu para nunca sugerir
let pinnedIds = new Set(); // ids fixados no topo das sugestões
let webIcons = true; // favicons dos sites conhecidos
let quick = {}; // palavra → item
let form = null;
let confirmId = null;
let qkTarget = null;
let winW = 860, winH = 840;
let lastWinH = 0;

// ───────────────────────── utilidades ─────────────────────────
function fz(q, t) {
  q = q.toLowerCase(); t = t.toLowerCase();
  if (q === t) return 1200;
  if (t.startsWith(q)) return 1000 - Math.min(t.length - q.length, 100);
  if (t.split(/[\s\-_.]+/).some((w) => w.startsWith(q))) return 850;
  if (t.includes(q)) return 600;
  let i = 0;
  for (const c of t) if (c === q[i]) i++;
  return i === q.length && q.length > 2 ? 300 : 0;
}

function fmtDur(s) {
  const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), x = Math.round(s % 60);
  return [h && `${h} h`, m && `${m} min`, x && `${x} s`].filter(Boolean).join(' ') || '0 s';
}
const fmtSize = (n) => { const u = ['B', 'KB', 'MB', 'GB', 'TB']; let i = 0; while (n >= 1024 && i < 4) { n /= 1024; i++; } return `${n.toFixed(i ? 1 : 0)} ${u[i]}`; };
const fmtDate = (t) => new Date(t * 1000).toLocaleString('pt-BR', { dateStyle: 'medium', timeStyle: 'short' });
const ago = (ts) => { const d = Date.now() / 1000 - ts; return d < 60 ? 'agora' : d < 3600 ? `há ${Math.floor(d / 60)} min` : d < 86400 ? `há ${Math.floor(d / 3600)} h` : new Date(ts * 1000).toLocaleDateString('pt-BR'); };

function toast(msg) {
  const el = document.createElement('div');
  el.className = 'toast';
  el.style.cssText = 'top:70px;bottom:auto;';
  el.textContent = msg;
  $('app').appendChild(el);
  setTimeout(() => el.remove(), 2200);
}

// ───────────────────────── vidro / janela ─────────────────────────
function offsetOf(el) {
  let x = 0, y = 0;
  while (el) { x += el.offsetLeft; y += el.offsetTop; el = el.offsetParent; }
  return { x, y };
}
function alignGlass() {
  document.querySelectorAll('.glass').forEach((g) => {
    if (g.hidden) return;
    const bg = g.querySelector('.glass-bg');
    const { x, y } = offsetOf(g);
    bg.style.width = winW + 'px';
    bg.style.height = winH + 'px';
    bg.style.transform = `translate(${-x + PAD}px, ${-y + PAD}px)`;
  });
}
// vidro com refração nas bordas (vidro translúcido)
document.querySelectorAll('.glass').forEach(attachLens);
function setBackdrop(seqNo) {
  document.documentElement.classList.toggle('nobd', !seqNo);
  const url = seqNo ? `url(${ICO}?bd=${seqNo})` : 'none';
  document.querySelectorAll('.glass-bg').forEach((b) => (b.style.backgroundImage = url));
}
let fitRaf = 0;
function fitWindow() {
  cancelAnimationFrame(fitRaf);
  fitRaf = requestAnimationFrame(() => {
    const h = Math.ceil($('app').offsetHeight + 40 + 64);
    if (Math.abs(h - lastWinH) > 1) { lastWinH = h; invoke('resize_window', { height: h }); }
    alignGlass();
  });
}
new ResizeObserver(fitWindow).observe($('app'));

// ───────────────────────── ícones de item ─────────────────────────
function tile(icon, color) { return `<span class="tile" style="--c:${color}">${G[icon]}</span>`; }
const PHOTO_RE = /\.(png|jpe?g|gif|webp|bmp|heic|avif|mp4|mkv|mov|avi|webm|pdf)$/i; // só estes ganham cantos/sombra de miniatura
function iconOf(it, big) {
  if (it.thumbSrc) return `<span class="ic photo ok"><img src="${it.thumbSrc}" alt="" draggable="false"></span>`;
  if (it.tile) return tile(it.tile, it.color);
  const size = big ? 128 : 96;
  let src;
  if (it.iconUrl) src = it.iconUrl;
  else if (it.kind === 'app') src = `${ICO}?a=${enc(it.path)}&s=${size}`;
  else src = `${ICO}?p=${enc(it.path)}&s=${size}&t=${it.thumb ? 1 : 0}`;
  const letter = esc((it.title || '?').trim()[0]?.toUpperCase() || '?');
  const photo = it.thumb && PHOTO_RE.test(it.path || '') ? ' photo' : '';
  return `<span class="ic${photo}"><b>${letter}</b><img src="${src}" alt="" draggable="false"></span>`;
}
document.addEventListener('load', (e) => { if (e.target.tagName === 'IMG') e.target.parentElement?.classList.add('ok'); }, true);
// Uma falha transitória do shell não deve deixar a letra para sempre: tenta uma vez mais.
document.addEventListener('error', (e) => {
  const img = e.target;
  if (img.tagName !== 'IMG' || img.dataset.retry) return;
  img.dataset.retry = '1';
  setTimeout(() => { img.src = img.src + '&r=1'; }, 250);
}, true);

// ───────────────────────── montagem de itens ─────────────────────────
function mapHit(h, opts = {}) {
  const kind = h.kind;
  const it = { id: h.id, kind, title: h.title, sub: kind === 'file' || kind === 'folder' ? h.subtitle : h.subtitle, path: h.path, score: h.score, mtime: h.mtime, thumb: opts.thumb };
  if (kind === 'setting') { it.tile = 'gear'; it.color = '#8e8e93'; }
  return it;
}
function actionItem(def, score = 500) {
  return { id: 'action:' + def.id, kind: 'action', actionId: def.id, def, title: def.title, sub: def.sub, tile: def.icon, color: def.color, score, danger: !!def.confirm };
}
// aplicativos ganham de pastas/arquivos de nome parecido: num lançador, digitar um nome costuma ser para abrir o programa
// ordem por tipo quando o texto casa de forma parecida: aplicativo, ação, pasta, arquivo (nome idêntico continua forte)
const TIER = { app: 250, action: 220, service: 230, folder: 60 };
function boost(it) { return (it.score || 0) + Math.min(usage[it.id] || 0, 10) * 30 + (TIER[it.kind] || 0); }

function localHits(q, onlyActions = false) {
  const out = [];
  let m;
  if (!onlyActions) out.push(...commandItems(q), ...serviceItems(q));
  if (!onlyActions) {
    const c = calculate(q);
    if (c) out.push({ id: 'calc', kind: 'calc', title: c, sub: `${q} =`, copy: c, tile: 'calc', color: '#ff9f0a', score: 1500 });
    else {
      const v = convert(q);
      if (v) out.push({ id: 'conv', kind: 'conv', title: v.title, sub: v.sub, copy: v.copy, tile: 'swap', color: '#30b0c7', score: 1500 });
    }
  }

  for (const def of ACTIONS) {
    const s = Math.max(fz(q, def.title), ...def.kw.map((k) => fz(q, k) - 20));
    if (s > 250) out.push(actionItem(def, s));
  }

  // linguagem natural
  if ((m = q.match(/^(?:timer|cron[oô]metro|temporizador|alarme)\s+(.+)$/i))) {
    const s = parseDuration(m[1]);
    if (s) out.push({ id: 'nlp:timer:' + s, kind: 'nlp', actionId: 'timer', args: { seconds: s, label: '' }, title: `Iniciar timer de ${fmtDur(s)}`, sub: 'Relógio', tile: 'timer', color: '#ff9f0a', score: 1450 });
  }
  if ((m = q.match(/^(?:nota|anotar|note)[:\s]+(.+)$/i))) {
    out.push({ id: 'nlp:note', kind: 'nlp', actionId: 'note', args: { title: m[1].slice(0, 40), text: m[1] }, title: `Criar nota: ${m[1]}`, sub: 'Notas', tile: 'note', color: '#ffd60a', score: 1440 });
  }
  if ((m = q.match(/^(?:evento|reuni[aã]o|lembrete|agendar)\s+(.+)$/i))) {
    const words = m[1].split(/\s+/);
    for (let k = Math.min(5, words.length - 1); k >= 1; k--) {
      const when = parseWhen(words.slice(-k).join(' '));
      if (when) {
        const title = words.slice(0, -k).join(' ');
        const end = new Date(when.getTime() + 3600e3);
        out.push({ id: 'nlp:event:' + title, kind: 'nlp', actionId: 'event', args: { title, start: toIcs(when), end: toIcs(end) }, title: `Evento: ${title}`, sub: `Calendário · ${describe(when)}`, tile: 'event', color: '#ff453a', score: 1430 });
        break;
      }
    }
  }

  const qk = quick[q.toLowerCase()];
  if (qk) out.push({ ...qk, def: qk.actionId ? A_BY_ID[qk.actionId] : undefined, score: 5000, viaQuick: true });
  return out;
}

function webItem(q) {
  const isUrl = /^https?:\/\//i.test(q) || (!/\s/.test(q) && /\.(com|org|net|io|dev|br|app|edu|gov)(\/|$)/i.test(q));
  const url = isUrl ? (/^https?:\/\//i.test(q) ? q : 'https://' + q) : `https://www.google.com/search?q=${enc(q)}`;
  return { id: 'web:' + q, kind: 'web', url, title: isUrl ? q : `Pesquisar "${q}" na Web`, sub: isUrl ? 'Abrir endereço no navegador' : 'Google', tile: 'globe', color: '#0a84ff', score: isUrl ? 950 : 50 };
}

function groupOf(it) { return GROUP[it.kind] || 'Outros'; }

function sectionize(list, { withTop = true } = {}) {
  // mesmo item vindo de duas fontes (ex.: atalho rápido + busca) aparece uma vez só
  const best = new Map();
  for (const it of list) if (!best.has(it.id) || boost(it) > boost(best.get(it.id))) best.set(it.id, it);
  const sorted = [...best.values()].sort((a, b) => boost(b) - boost(a));
  const sections = [];
  if (withTop && sorted.length && boost(sorted[0]) >= 750 && !['calc', 'conv', 'web'].includes(sorted[0].kind)) {
    sections.push({ title: 'Melhor resultado', items: [sorted.shift()], top: true });
  }
  const map = new Map();
  for (const it of sorted) {
    const g = groupOf(it);
    if (!map.has(g)) map.set(g, []);
    map.get(g).push(it);
  }
  const rest = [...map.entries()].sort((a, b) => {
    const wa = a[0] === 'Pesquisar na web' ? -1 : boost(a[1][0]);
    const wb = b[0] === 'Pesquisar na web' ? -1 : boost(b[1][0]);
    return wb - wa;
  });
  for (const [title, its] of rest) sections.push({ title, items: its });
  return sections;
}


// ───────────────────────── comandos digitados ─────────────────────────
// ssh, rdp, ping, pasta de rede, IP e "navegador pesquisa". Nada roda sem Enter; o comando exato aparece no item.
let sshHosts = [];
const NAME = '[A-Za-z0-9][A-Za-z0-9._-]*';
const BROWSERS = { brave: 'Brave', chrome: 'Chrome', firefox: 'Firefox', edge: 'Edge', msedge: 'Edge', opera: 'Opera', vivaldi: 'Vivaldi' };
const cmdItem = (id, cmd, title, sub, tile, color, score = 1400) => ({ id: 'cmd:' + id, kind: 'cmd', cmd, title, sub, tile, color, score });

function commandItems(q) {
  const out = [];
  let m;
  if ((m = q.match(new RegExp(`^ssh\\s+(?:([A-Za-z0-9._-]{1,64})@)?(${NAME})(?::(\\d{1,5}))?$`, 'i')))) {
    const [, user = '', host, port = ''] = m;
    const shown = `${user ? user + '@' : ''}${host}`;
    out.push(cmdItem(`ssh:${shown}:${port}`, { kind: 'ssh', user, host, port: +port || 0 }, `Conectar por SSH a ${shown}${port ? ':' + port : ''}`, `Abre o terminal com: ssh ${port ? '-p ' + port + ' ' : ''}${shown}`, 'term', '#1c1c1e'));
  } else if ((m = q.match(/^ssh\s+(\S*)$/i))) {
    for (const h of sshHosts.filter((x) => x.toLowerCase().startsWith(m[1].toLowerCase())).slice(0, 6)) {
      out.push(cmdItem(`sshalias:${h}`, { kind: 'ssh', user: '', host: h, port: 0 }, `Conectar por SSH a ${h}`, `Do seu ~/.ssh/config · ssh ${h}`, 'term', '#1c1c1e', 1300));
    }
  }
  if ((m = q.match(new RegExp(`^rdp\\s+(${NAME})(?::(\\d{1,5}))?$`, 'i'))) && IS_WIN) {
    out.push(cmdItem(`rdp:${m[1]}:${m[2] || ''}`, { kind: 'rdp', host: m[1], port: +m[2] || 0 }, `Área de Trabalho Remota em ${m[1]}`, `Abre o mstsc /v:${m[1]}${m[2] ? ':' + m[2] : ''}`, 'term', '#0a84ff'));
  }
  if ((m = q.match(new RegExp(`^ping\\s+(${NAME})$`, 'i')))) {
    out.push(cmdItem(`ping:${m[1]}`, { kind: 'ping', host: m[1] }, `Ping em ${m[1]}`, `Abre o terminal com: ping ${m[1]}`, 'term', '#1c1c1e'));
  }
  if ((m = q.match(/^(?:\\\\|\/\/)([A-Za-z0-9][A-Za-z0-9._-]*)((?:[\\/][^\\/:*?"<>|]+)*)[\\/]?$/))) {
    const path = '\\\\' + m[1] + m[2].replace(/\//g, '\\');
    out.push(cmdItem('unc:' + path, { kind: 'unc', path }, 'Abrir pasta de rede', path, 'folder', '#ff9f0a'));
  }
  if ((m = q.match(/^(\d{1,3}(?:\.\d{1,3}){3})(:\d{1,5})?(\/\S*)?$/)) && m[1].split('.').every((o) => +o <= 255)) {
    const url = 'http://' + q;
    out.push(cmdItem('url:' + url, { kind: 'url', url }, `Abrir ${url} no navegador`, 'Endereço de rede', 'globe', '#0a84ff'));
  }
  if ((m = q.match(/^(brave|chrome|firefox|edge|msedge|opera|vivaldi)\s+(\S.*)$/i))) {
    const b = m[1].toLowerCase(), rest = m[2].trim(), name = BROWSERS[b];
    const isUrl = /^https?:\/\/\S+$/i.test(rest) || /^[\w-]+(\.[\w-]+)+(:\d+)?(\/\S*)?$/.test(rest);
    const url = isUrl ? (/^https?:\/\//i.test(rest) ? rest : 'https://' + rest) : `https://www.google.com/search?q=${enc(rest)}`;
    out.push(cmdItem(`browser:${b}:${rest}`, { kind: 'browser', browser: b, url }, isUrl ? `Abrir ${rest} no ${name}` : `Pesquisar "${rest}" no ${name}`, `Abre o ${name} direto`, 'globe', '#0a84ff', 1420));
  }
  return out;
}

function serviceItems(q) {
  if (q.length < 3 || q.length > 24 || /\s{2,}/.test(q)) return [];
  const out = [];
  for (const s of SERVICES) {
    const sc = Math.max(fz(q, s.n.toLowerCase()), ...s.k.map((k) => fz(q, k)));
    if (sc < 800) continue; // só nome/apelido que COMEÇA igual: nada de letras espalhadas
    out.push({ id: 'service:' + s.d, kind: 'service', url: s.u, title: s.n, sub: `Abrir ${new URL(s.u).host} no navegador`, iconUrl: webIcons ? `https://www.google.com/s2/favicons?domain=${s.d}&sz=64` : '', tile: webIcons ? undefined : 'globe', color: '#0a84ff', score: Math.min(1350, sc + 250) });
  }
  return out;
}

// ───────────────────────── busca ─────────────────────────
async function backend(q, f) {
  try { return (await invoke('search', { query: q, filter: f })).map((h) => mapHit(h, { thumb: false })); }
  catch (e) { console.error(e); return []; }
}

async function compute() {
  const q = input.value.trim();

  if (filter === 'clip') {
    const list = (await invoke('clip_list')).filter((c) => !q || c.text.toLowerCase().includes(q.toLowerCase()));
    const its = list.map((c) => {
      const base = { id: 'clip:' + c.id, kind: 'clip', clipId: c.id, score: 0 };
      if (c.kind === 'image') return { ...base, title: 'Imagem', sub: `${c.w}×${c.h} · ${ago(c.ts)}`, thumbSrc: `${ICO}?c=${c.id}&th=1` };
      if (c.kind === 'files') return { ...base, title: c.text.slice(0, 120), sub: `${c.files.length} ${c.files.length > 1 ? 'arquivos' : 'arquivo'} · ${ago(c.ts)}`, path: c.files[0], thumb: true };
      return { ...base, title: c.text.trim().split('\n')[0].slice(0, 120), sub: `${ago(c.ts)} · ${c.text.length} caracteres`, tile: 'clip', color: '#8e8e93' };
    });
    return { head: { icon: 'clip', title: 'Área de transferência', extra: its.length ? '<button class="pbtn" id="clipclear">Limpar tudo</button>' : '' }, sections: its.length ? [{ title: null, items: its }] : [], layout: 'list', empty: 'Nenhum resultado' };
  }

  if (filter === 'apps') {
    if (!q) {
      const apps = (await invoke('list_apps')).map((h) => mapHit(h));
      // os mais abertos primeiro (não ordem alfabética); Delete remove um item das sugestões
      const pinned = apps.filter((a) => pinnedIds.has(a.id)).map((a) => ({ ...a, pinned: true }));
      const top = apps.filter((a) => (usage[a.id] || 0) > 0 && !neverSuggest.has(a.id) && !pinnedIds.has(a.id)).sort((a, b) => usage[b.id] - usage[a.id]);
      const used = [...pinned, ...top].slice(0, Math.max(6, pinned.length));
      const sections = [];
      if (used.length) sections.push({ title: 'Sugestões', hint: 'botão direito: fixar · <kbd>Del</kbd> nunca sugerir', items: used });
      sections.push({ title: 'Aplicativos', items: apps });
      return { head: { icon: 'apps', title: 'Aplicativos' }, sections, layout: 'grid', cols: 6, empty: 'Nenhum aplicativo' };
    }
    const hits = (await backend(q, 'apps')).sort((a, b) => boost(b) - boost(a));
    return { head: { icon: 'apps', title: 'Aplicativos' }, sections: hits.length ? [{ title: null, items: hits }] : [], layout: 'grid', cols: 6, empty: 'Nenhum resultado' };
  }

  if (filter === 'files') {
    const chips = FILE_CHIPS.map(([l, k]) => `<button class="chip ${k === fileKind ? 'on' : ''}" data-k="${k}">${l}</button>`).join('');
    if (!q) {
      const recent = (await invoke('recent', { kind: fileKind, limit: 54 })).map((h) => mapHit(h, { thumb: true }));
      const sections = [];
      if (recent.length) {
        sections.push({ title: 'Sugestões', items: recent.slice(0, 10) });
        if (recent.length > 10) sections.push({ title: 'Recentes', items: recent.slice(10) });
      }
      return { head: { icon: 'folder', title: 'Arquivos' }, chips, sections, layout: 'grid', cols: 5, gridClass: 'files', empty: 'Nenhum arquivo recente' };
    }
    const hits = await backend(q, 'files');
    const sections = sectionize(hits, { withTop: false });
    return { head: { icon: 'folder', title: 'Arquivos' }, chips: null, sections, layout: 'list', empty: 'Nenhum resultado' };
  }

  if (filter === 'actions') {
    if (!q) {
      const settings = await backend('', 'actions');
      const sections = [
        { title: 'Sugestões', items: ACTIONS.filter((a) => a.fields).map((a) => actionItem(a)) },
        { title: 'Sistema', items: ACTIONS.filter((a) => !a.fields).map((a) => actionItem(a)) },
      ];
      if (settings.length) sections.push({ title: 'Configurações', items: settings });
      return { head: { icon: 'bolt', title: 'Ações' }, sections, layout: 'list' };
    }
    const sections = sectionize([...localHits(q, true), ...(await backend(q, 'actions'))], { withTop: false });
    return { head: { icon: 'bolt', title: 'Ações' }, sections, layout: 'list', empty: 'Nenhum resultado' };
  }

  // modo geral
  const [remote] = await Promise.all([backend(q, 'all')]);
  const all = [...localHits(q), ...remote, webItem(q)];
  return { head: null, sections: sectionize(all), layout: 'list', empty: 'Nenhum resultado' };
}

async function refresh() {
  if (form || settingsOpen) return;
  const my = ++seq;
  const q = input.value.trim();
  if (!q && !filter) { showPanel(false); return; }
  const res = await compute();
  if (my !== seq) return;
  render(res);
}

// ───────────────────────── renderização ─────────────────────────
function rowHtml(it, i, top) {
  const qk = Object.entries(quick).find(([, v]) => v.id === it.id)?.[0];
  let right = '';
  if (it.kind === 'clip') right = `<button class="pbtn" data-del="${it.clipId}" title="Apagar (Del)">✕</button>`;
  else if (['app', 'file', 'folder', 'setting', 'action'].includes(it.kind)) right = qk ? `<kbd>${esc(qk)}</kbd>` : `<span class="qkhint"><kbd>Ctrl+K</kbd> atalho</span>`;
  else if (it.kind === 'calc' || it.kind === 'conv') right = '<kbd>↵ copiar</kbd>';
  const isConfirm = confirmId === it.id;
  return `<div class="row ${top ? 'top' : ''} ${it.danger ? 'danger' : ''}" data-i="${i}">${iconOf(it, top)}<div class="txt"><div class="t ${it.kind === 'calc' || it.kind === 'conv' ? 'big' : ''}">${esc(it.title)}</div><div class="s">${isConfirm ? '<b>Pressione Enter novamente para confirmar</b>' : esc(it.sub || '')}</div></div><div class="r">${right}</div></div>`;
}
function cellHtml(it, i) {
  return `<div class="cell${it.pinned ? ' pinned' : ''}" data-i="${i}" title="${esc(it.title)}">${iconOf(it)}<div class="lab">${esc(it.title.replace(/\.[^.]{1,5}$/, it.kind === 'file' ? '' : '$&'))}</div></div>`;
}

function render(res) {
  items = [];
  cols = res.layout === 'grid' ? res.cols : 1;

  // cabeçalho
  phead.innerHTML = res.head ? `<span class="hico">${G[res.head.icon]}</span><span>${esc(res.head.title)}</span><span class="spacer"></span>${res.head.extra || ''}` : '';
  chipsEl.innerHTML = res.chips || '';

  let html = '';
  for (const s of res.sections) {
    if (s.title && !(res.sections.length === 1 && res.head && res.head.title === s.title)) html += `<div class="sec"><span>${esc(s.title)}</span>${s.hint ? `<span class="more">${s.hint}</span>` : ''}</div>`;
    if (res.layout === 'grid') {
      html += `<div class="grid ${res.gridClass || ''}" style="grid-template-columns:repeat(${res.cols},1fr)">`;
      for (const it of s.items) { html += cellHtml(it, items.length); items.push(it); }
      html += '</div>';
    } else {
      for (const it of s.items) { html += rowHtml(it, items.length, s.top); items.push(it); }
    }
  }
  if (!items.length) html = `<div class="empty">${esc(res.empty || 'Nenhum resultado')}</div>`;
  pbody.innerHTML = html;
  pbody.scrollTop = 0;
  sel = 0;
  showPanel(true);
  setSel(0, false);
}

function showPanel(on) {
  if (on && panel.hidden) { panel.hidden = false; panel.classList.remove('open'); void panel.offsetWidth; panel.classList.add('open'); }
  if (!on) { panel.hidden = true; items = []; }
  alignGlass();
  fitWindow();
}

function setSel(i, scroll = true) {
  if (!items.length) return;
  sel = Math.max(0, Math.min(items.length - 1, i));
  pbody.querySelector('.sel')?.classList.remove('sel');
  const el = pbody.querySelector(`[data-i="${sel}"]`);
  if (el) { el.classList.add('sel'); if (scroll) el.scrollIntoView({ block: 'nearest' }); }
}

// ───────────────────────── ações ─────────────────────────
const hide = () => invoke('hide_window');

async function runAction(id, args) {
  try { const msg = await invoke('run_action', { id, args }); if (msg) toast(msg); return true; }
  catch (e) { toast(String(e)); return false; }
}

async function activate(it, ctrl = false) {
  if (!it) return;
  switch (it.kind) {
    case 'app': case 'file': case 'folder': case 'setting':
      if (ctrl && (it.kind === 'file' || it.kind === 'folder')) await invoke('reveal', { path: it.path });
      else await invoke('open_item', { id: it.id, kind: it.kind, path: it.path });
      hide();
      break;
    case 'action': {
      const def = it.def || A_BY_ID[it.actionId];
      if (def.id === 'prefs') { openSettings(); return; }
      if (def.fields) { enterForm(def); return; }
      if (def.confirm && confirmId !== it.id) { confirmId = it.id; refreshRows(); return; }
      if (await runAction(def.id, {})) hide();
      break;
    }
    case 'cmd':
      if (it.cmd.kind === 'url') await runAction('url', { url: it.cmd.url });
      else { try { await invoke('run_cmd', { args: it.cmd }); } catch (e) { toast(String(e)); return; } }
      hide();
      break;
    case 'nlp':
      if (await runAction(it.actionId, it.args)) hide();
      break;
    case 'calc': case 'conv':
      await invoke('copy_text', { text: it.copy });
      hide();
      break;
    case 'service':
    case 'web':
      await runAction('url', { url: it.url });
      hide();
      break;
    case 'clip':
      await invoke('clip_copy', { id: it.clipId });
      hide();
      break;
  }
}
function refreshRows() { const keep = sel; const q = input.value; refresh().then(() => setSel(keep)); void q; }

// ───────────────────────── formulário em linha (linguagem natural) ─────────────────────────
function enterForm(def) {
  form = { def };
  input.hidden = true;
  formEl.hidden = false;
  formEl.innerHTML = def.tpl.map((p) => (def.fields[p] !== undefined
    ? `<input class="tok" data-k="${p}" placeholder="${esc(def.fields[p])}" spellcheck="false" />`
    : `<span class="lbl">${esc(p)}</span>`)).join('');
  const toks = [...formEl.querySelectorAll('.tok')];
  toks.forEach((t) => {
    t.addEventListener('input', updateFormHint);
    t.addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); e.stopPropagation(); submitForm(); } });
  });
  toks[0]?.focus();
  updateFormHint();
}
function exitForm() {
  form = null;
  formEl.hidden = true; formEl.innerHTML = '';
  input.hidden = false;
  input.focus();
  refresh();
}
const formVals = () => Object.fromEntries([...formEl.querySelectorAll('.tok')].map((t) => [t.dataset.k, t.value.trim()]));

function formPayload(def, v) {
  switch (def.id) {
    case 'timer': {
      const s = parseDuration(v.dur || '');
      if (!s) throw 'Informe uma duração, ex.: 10 minutos';
      return { seconds: s, label: v.label || '' };
    }
    case 'note': return { title: v.title, text: v.text };
    case 'event': {
      const start = parseWhen(v.start || '');
      if (!start) throw 'Não entendi o início. Ex.: amanhã 14h';
      let end = v.end ? parseWhen(v.end, start) : null;
      if (!end || end <= start) end = new Date(start.getTime() + 3600e3);
      return { title: v.title || 'Evento', start: toIcs(start), end: toIcs(end) };
    }
    case 'email': return { to: v.to, subject: v.subject, body: '' };
    case 'message': return { text: v.text };
  }
  return {};
}
function updateFormHint() {
  const { def } = form;
  const v = formVals();
  let lines = [];
  try {
    if (def.id === 'timer') { const s = parseDuration(v.dur || ''); lines.push(s ? `Duração: <b>${fmtDur(s)}</b>` : 'Ex.: 10 minutos, 1h30, 45 s'); }
    else if (def.id === 'event') {
      const s = parseWhen(v.start || ''); const e = s && v.end ? parseWhen(v.end, s) : null;
      lines.push(s ? `Início: <b>${describe(s)}</b>` : 'Ex.: amanhã 14h, sexta 18:30, 15/11 9h');
      if (s) lines.push(`Fim: <b>${describe(e && e > s ? e : new Date(s.getTime() + 3600e3))}</b>`);
    } else if (def.id === 'note') lines.push('A nota é salva em Documentos\\Notas e aberta no editor padrão.');
    else if (def.id === 'email') lines.push('Abre o seu aplicativo de e-mail padrão.');
    else if (def.id === 'message') lines.push('Abre o WhatsApp com o texto pronto.');
  } catch {}
  phead.innerHTML = `<span class="hico">${G[def.icon]}</span><span>${esc(def.title)}</span><span class="spacer"></span><span style="font-weight:400;font-size:12px;color:var(--fg2)"><kbd>↵</kbd> executar &nbsp; <kbd>Esc</kbd> cancelar</span>`;
  chipsEl.innerHTML = '';
  pbody.innerHTML = `<div class="empty" style="padding:14px 0 18px;line-height:1.7">${lines.join('<br>')}</div>`;
  items = [];
  showPanel(true);
}
async function submitForm() {
  const { def } = form;
  let payload;
  try { payload = formPayload(def, formVals()); } catch (e) { toast(String(e)); return; }
  if (await runAction(def.id, payload)) { form = null; hide(); }
}

// ───────────────────────── atalho rápido (Ctrl+K) ─────────────────────────
function openQuickKey() {
  const it = items[sel];
  if (!it || !['app', 'file', 'folder', 'setting', 'action'].includes(it.kind)) return;
  qkTarget = it;
  const cur = Object.entries(quick).find(([, v]) => v.id === it.id)?.[0] || '';
  qkbar.hidden = false;
  qkbar.innerHTML = `<span>Atalho rápido para <b>${esc(it.title)}</b>:</span><input id="qkin" maxlength="12" placeholder="ex.: ch" value="${esc(cur)}" /><span style="color:var(--fg3)">↵ salvar · vazio remove</span>`;
  const inp = $('qkin');
  inp.focus(); inp.select();
  inp.addEventListener('keydown', async (e) => {
    e.stopPropagation();
    if (e.key === 'Escape') { closeQuickKey(); }
    if (e.key === 'Enter') {
      const word = inp.value.trim().toLowerCase();
      if (cur) await invoke('quick_remove', { keyword: cur });
      if (word) {
        const t = qkTarget;
        await invoke('quick_set', { keyword: word, item: { id: t.id, kind: t.kind, title: t.title, sub: t.sub, path: t.path, actionId: t.actionId, tile: t.tile, color: t.color, danger: t.danger } });
        toast(`Digite "${word}" para abrir ${t.title}`);
      }
      quick = await invoke('quick_get');
      closeQuickKey(); refresh();
    }
  });
  fitWindow();
}
function closeQuickKey() { qkbar.hidden = true; qkbar.innerHTML = ''; qkTarget = null; input.focus(); fitWindow(); }

// ───────────────────────── Prévia rápida ─────────────────────────
async function openPreview() {
  const it = items[sel];
  if (!it || !['file', 'folder', 'app'].includes(it.kind)) return;
  let meta = '';
  if (it.kind !== 'app') { try { const i = await invoke('file_info', { path: it.path }); meta = `${i.dir ? 'Pasta' : fmtSize(i.size)} · Modificado ${fmtDate(i.mtime)}`; } catch {} }
  const img = it.kind === 'app' ? `${ICO}?a=${enc(it.path)}&s=256` : `${ICO}?p=${enc(it.path)}&s=512&t=1`;
  ql.hidden = false;
  ql.innerHTML = `<img src="${img}" alt=""><div class="qn">${esc(it.title)}</div><div class="qm">${esc(it.kind === 'app' ? 'Aplicativo' : it.path)}</div><div class="qm">${esc(meta)}</div><div class="qa"><button class="pri" id="qlopen">Abrir</button>${it.kind !== 'app' ? '<button id="qlrev">Mostrar na pasta</button>' : ''}<button id="qlclose">Fechar</button></div>`;
  $('qlopen').onclick = () => activate(it);
  $('qlrev') && ($('qlrev').onclick = () => invoke('reveal', { path: it.path }).then(hide));
  $('qlclose').onclick = closePreview;
}
function closePreview() { ql.hidden = true; ql.innerHTML = ''; input.focus(); }



// ───────────────────────── novas versões ─────────────────────────
// Aviso simples: consulta a última release no GitHub (1x por dia, desligável) e oferece abrir a página de download.
const REPO = 'splt5k/glint-search';
let appVersion = '';
let updateInfo = null; // { version, url } quando há versão mais nova
let checkUpdates = true;
const isNewer = (a, b) => {
  const p = (s) => s.replace(/^v/, '').split('.').map((n) => parseInt(n, 10) || 0);
  const [x, y] = [p(a), p(b)];
  for (let i = 0; i < 3; i++) if ((x[i] || 0) !== (y[i] || 0)) return (x[i] || 0) > (y[i] || 0);
  return false;
};
let canSelfUpdate = false; // true quando o atualizador assinado achou a versão (Windows/AppImage)
async function checkUpdate() {
  try {
    if (!appVersion) appVersion = await T.app.getVersion();
    canSelfUpdate = false;
    try {
      const u = await invoke('update_check');
      if (u) { updateInfo = { version: u.version, url: `https://github.com/${REPO}/releases/latest` }; canSelfUpdate = true; try { localStorage.setItem('glint.lastCheck', String(Date.now())); } catch {} return 'new'; }
    } catch { /* sem suporte a atualização automática aqui: usa o aviso pelo GitHub */ }
    const r = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, { headers: { Accept: 'application/vnd.github+json' } });
    if (!r.ok) return 'err';
    const j = await r.json();
    updateInfo = isNewer(j.tag_name, appVersion) ? { version: j.tag_name.replace(/^v/, ''), url: j.html_url } : null;
    try { localStorage.setItem('glint.lastCheck', String(Date.now())); } catch {}
    return updateInfo ? 'new' : 'ok';
  } catch { return 'err'; }
}
function maybeCheckUpdate() {
  if (!checkUpdates) return;
  let last = 0;
  try { last = +localStorage.getItem('glint.lastCheck') || 0; } catch {}
  if (Date.now() - last < 24 * 3600e3) return;
  checkUpdate().then((r) => { if (r === 'new') toast(`Glint ${updateInfo.version} disponível: veja em Configurações (Ctrl+,)`); });
}

// ───────────────────────── configurações ─────────────────────────
let settingsOpen = false;
let cfg = null; // cópia editável (só vai para o disco ao salvar)

const KEY_NAMES = { Space: 'Espaço', Escape: 'Esc', Enter: 'Enter', Super: IS_WIN ? 'Win' : 'Super' };
const prettyKey = (hk) => hk.split('+').map((k) => KEY_NAMES[k] || k.replace(/^Key|^Digit/, '')).join(' + ');
function hotkeyFrom(e) {
  if (/^(Control|Shift|Alt|Meta)(Left|Right)$/.test(e.code)) return null;
  const mods = [e.ctrlKey && 'Ctrl', e.altKey && 'Alt', e.shiftKey && 'Shift', e.metaKey && 'Super'].filter(Boolean);
  if (!mods.length && !/^F\d+$/.test(e.code)) return undefined; // precisa de modificador (ou tecla F)
  return [...mods, e.code].join('+');
}
const lines = (t) => t.split('\n').map((x) => x.trim()).filter(Boolean);

async function openSettings() {
  const meta = await invoke('config_get');
  cfg = meta.config;
  settingsOpen = true;
  if (form) exitForm();
  qkbar.hidden = true; ql.hidden = true;
  input.value = ''; clearBtn.hidden = true;
  document.querySelectorAll('.cat').forEach((b) => b.classList.remove('on')); filter = null;
  renderSettings(meta);
}

function renderSettings(meta) {
  const chk = (id, label, on, hint = '') => `<label class="set-row"><input type="checkbox" id="${id}" ${on ? 'checked' : ''}><span><b>${label}</b>${hint ? `<small>${hint}</small>` : ''}</span></label>`;
  const warn = !meta.hotkeyOk ? `<div class="set-warn">O atalho atual não pôde ser registrado (outro programa o usa). Escolha outro abaixo.</div>` : '';
  const way = meta.wayland ? `<div class="set-warn">No Wayland o sistema não permite atalhos globais para aplicativos. Crie um atalho do sistema (Configurações › Teclado) que execute o comando <kbd>glint</kbd>: ele abre esta janela.</div>` : '';
  phead.innerHTML = `<span class="hico">${G.gear}</span><span>Configurações</span><span class="spacer"></span><span style="font-weight:400;font-size:12px;color:var(--fg2)"><kbd>Esc</kbd> voltar</span>`;
  chipsEl.innerHTML = '';
  pbody.innerHTML = `<div class="settings">
    ${warn}${way}
    <div class="set-block"><div class="set-lbl">Atalho para abrir</div>
      <div class="set-line"><input id="cfHotkey" readonly value="${esc(prettyKey(cfg.hotkey))}" title="Clique e pressione a combinação"><button class="pbtn" id="cfHkReset">Padrão</button></div>
      <small>Clique no campo e pressione a combinação desejada (ex.: Ctrl + Alt + Espaço).</small></div>
    <div class="set-block">
      ${chk('cfAuto', 'Iniciar com o sistema', cfg.autostart)}
      ${chk('cfDrives', 'Indexar todos os discos', cfg.all_drives, 'Desligado: só Documentos, Downloads, Imagens etc. e as pastas extras.')}
      ${chk('cfBat', 'Pausar a indexação completa na bateria', cfg.pause_on_battery)}
      ${chk('cfUpd', 'Avisar quando houver versão nova (consulta o GitHub 1x por dia)', cfg.check_updates)}
      ${chk('cfIcons', 'Mostrar o ícone (favicon) dos sites conhecidos', cfg.web_icons, 'Baixado do serviço de ícones do Google, só para os sites da lista interna (Gmail, YouTube...).')}
    </div>
    <div class="set-block"><div class="set-lbl">Tema</div>
      <select id="cfTheme"><option value="system">Seguir o sistema</option><option value="light">Claro</option><option value="dark">Escuro</option></select></div>
    <div class="set-block"><div class="set-lbl">Pastas extras para indexar <small>(uma por linha)</small></div><textarea id="cfInc" rows="2" spellcheck="false">${esc(cfg.include.join('\n'))}</textarea></div>
    <div class="set-block"><div class="set-lbl">Pastas ignoradas <small>(uma por linha)</small></div><textarea id="cfExc" rows="2" spellcheck="false">${esc(cfg.exclude.join('\n'))}</textarea></div>
    <div class="set-block"><div class="set-lbl">Nunca sugerir <small>(itens ocultos das Sugestões; use Delete numa sugestão)</small></div>
      <div class="set-hidden">${cfg.never_suggest.length ? cfg.never_suggest.map((h) => `<span class="chip">${esc(h.title)} <button data-unhide="${esc(h.id)}" title="Voltar a sugerir">✕</button></span>`).join('') : '<small>Nenhum.</small>'}</div></div>
    <div class="set-block"><div class="set-lbl">Versão <small id="cfVer"></small></div>
      <div class="set-line"><button class="pbtn" id="cfCheck">Verificar agora</button><button class="pbtn pri" id="cfGet" hidden>Baixar</button></div></div>
    <div class="set-actions">
      <button class="pbtn" id="cfExport">Exportar…</button><button class="pbtn" id="cfImport">Importar…</button><button class="pbtn" id="cfReindex">Reindexar agora</button><button class="pbtn" id="cfForget" title="Zera as sugestões e o aprendizado de uso">Limpar sugestões</button>
      <span class="spacer"></span><button class="pbtn pri" id="cfSave">Salvar</button>
    </div>
    <div class="set-msg" id="cfMsg"></div>
  </div>`;
  $('cfTheme').value = cfg.theme;
  items = []; cols = 1;
  showPanel(true);
  wireSettings(meta);
}

function collectSettings() {
  return { ...cfg, autostart: $('cfAuto').checked, all_drives: $('cfDrives').checked, pause_on_battery: $('cfBat').checked, web_icons: $('cfIcons').checked, check_updates: $('cfUpd').checked, theme: $('cfTheme').value, include: lines($('cfInc').value), exclude: lines($('cfExc').value) };
}
const setMsg = (t, bad) => { const m = $('cfMsg'); if (m) { m.textContent = t; m.className = 'set-msg' + (bad ? ' bad' : ''); } };

function wireSettings(meta) {
  const hk = $('cfHotkey');
  hk.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' || (e.key === 'Tab' && !e.ctrlKey && !e.altKey && !e.metaKey)) return; // fechar / navegar com Tab
    if (!e.code) return;
    e.preventDefault(); e.stopPropagation();
    const v = hotkeyFrom(e);
    if (v === null) return;
    if (v === undefined) return setMsg('Use ao menos Ctrl, Alt, Shift ou Win junto com a tecla.', true);
    cfg.hotkey = v; hk.value = prettyKey(v); setMsg('');
  });
  $('cfHkReset').addEventListener('click', () => { cfg.hotkey = 'Ctrl+Space'; hk.value = prettyKey(cfg.hotkey); });
  $('cfSave').addEventListener('click', async () => {
    try { const m = await invoke('config_set', { config: collectSettings() }); cfg = m.config; neverSuggest = new Set(cfg.never_suggest.map((h) => h.id)); webIcons = cfg.web_icons !== false; checkUpdates = cfg.check_updates !== false; applyThemePref(); setMsg('Salvo.'); }
    catch (e) { setMsg(String(e), true); }
  });
  $('cfExport').addEventListener('click', async () => {
    try { const p = await invoke('config_export'); if (p) setMsg('Exportado para ' + p); } catch (e) { setMsg(String(e), true); }
  });
  $('cfImport').addEventListener('click', async () => {
    try {
      const c = await invoke('config_import');
      if (c) { cfg = c; renderSettings({ ...meta, hotkeyOk: true, wayland: meta.wayland }); setMsg('Importado. Revise e clique em Salvar para aplicar.'); }
    } catch (e) { setMsg(String(e), true); }
  });
  pbody.querySelectorAll('[data-unhide]').forEach((b) => b.addEventListener('click', () => { cfg.never_suggest = cfg.never_suggest.filter((h) => h.id !== b.dataset.unhide); b.closest('.chip').remove(); setMsg('Clique em Salvar para aplicar.'); }));
  const verLine = () => { $('cfVer').textContent = updateInfo ? `v${appVersion} · nova: v${updateInfo.version}` : `v${appVersion || '?'}`; $('cfGet').hidden = !updateInfo; if (updateInfo) $('cfGet').textContent = canSelfUpdate ? `Atualizar para v${updateInfo.version}` : `Baixar v${updateInfo.version}`; };
  T.app.getVersion().then((v) => { appVersion = v; verLine(); }).catch(() => {});
  $('cfCheck').addEventListener('click', async () => { setMsg('Verificando…'); const r = await checkUpdate(); verLine(); setMsg(r === 'new' ? `Versão ${updateInfo.version} disponível.` : r === 'ok' ? 'Você está na versão mais recente.' : 'Não consegui consultar o GitHub agora.', r === 'err'); });
  $('cfGet').addEventListener('click', async () => {
    if (!canSelfUpdate) return runAction('url', { url: updateInfo.url });
    setMsg('Baixando e instalando… o Glint vai reiniciar.');
    try { await invoke('update_install'); } catch (e) { setMsg(String(e), true); }
  });
  $('cfForget').addEventListener('click', async () => { await invoke('usage_forget', { id: null }); usage = {}; setMsg('Sugestões e histórico de uso limpos.'); });
  $('cfReindex').addEventListener('click', async () => { await invoke('reindex_now'); setMsg('Reindexando em segundo plano…'); });
  pbody.querySelectorAll('input,textarea,select').forEach((el) => el.addEventListener('keydown', (e) => { if (e.key !== 'Escape') e.stopPropagation(); }));
}

// o tema escolhido vale já na próxima abertura; aqui só refletimos na hora
async function applyThemePref() {
  const t = cfg.theme;
  if (t === 'light' || t === 'dark') document.documentElement.dataset.theme = t;
}

function closeSettings() {
  settingsOpen = false;
  input.focus();
  showPanel(false);
  refresh();
}


// ───────────────────────── menu do botão direito ─────────────────────────
const ctxEl = document.createElement('div');
ctxEl.id = 'ctx';
ctxEl.hidden = true;
document.body.appendChild(ctxEl);
let ctxActions = [];
const isWindows = IS_WIN;

function hideCtx() { ctxEl.hidden = true; ctxActions = []; }

function ctxEntries(it) {
  const E = [];
  const open = ['Abrir', () => activate(it)];
  if (it.kind === 'app') {
    const pinned = pinnedIds.has(it.id);
    E.push(open, null,
      [pinned ? 'Desafixar das sugestões' : 'Fixar nas sugestões', async () => {
        await invoke('suggest_pin', { id: it.id, title: it.title, pin: !pinned });
        if (pinned) pinnedIds.delete(it.id); else { pinnedIds.add(it.id); neverSuggest.delete(it.id); }
        toast(pinned ? 'Removido dos fixados' : `${it.title} fixado nas sugestões`);
        refresh();
      }],
      ['Nunca sugerir', async () => {
        await invoke('suggest_hide', { id: it.id, title: it.title });
        neverSuggest.add(it.id); pinnedIds.delete(it.id); delete usage[it.id];
        toast(`${it.title} nunca mais aparece nas sugestões`); refresh();
      }],
      null,
      ['Abrir local do arquivo', async () => { try { await invoke('app_reveal', { id: it.path }); hide(); } catch (e) { toast(String(e)); } }]);
    if (isWindows) E.push(['Desinstalar…', async () => { try { toast(await invoke('app_uninstall', { name: it.title })); } catch (e) { toast(String(e)); } }]);
  } else if (it.kind === 'file' || it.kind === 'folder') {
    E.push(open, ['Mostrar na pasta', async () => { await invoke('reveal', { path: it.path }); hide(); }], null,
      ['Copiar caminho', async () => { await invoke('copy_text', { text: it.path }); toast('Caminho copiado'); }]);
  } else if (it.kind === 'clip') {
    E.push(['Copiar', () => activate(it)], ['Apagar', async () => { await invoke('clip_delete', { id: it.clipId }); refresh(); }]);
  } else {
    E.push(open);
  }
  return E;
}

function openCtx(it, x, y) {
  const entries = ctxEntries(it);
  ctxActions = entries.filter(Boolean).map((e) => e[1]);
  let n = 0;
  ctxEl.innerHTML = entries.map((e) => (e ? `<button data-n="${n++}">${esc(e[0])}</button>` : '<hr>')).join('');
  ctxEl.hidden = false;
  const r = ctxEl.getBoundingClientRect();
  ctxEl.style.left = Math.max(4, Math.min(x, window.innerWidth - r.width - 4)) + 'px';
  ctxEl.style.top = Math.max(4, Math.min(y, window.innerHeight - r.height - 4)) + 'px';
  ctxEl.querySelector('button')?.focus();
}

// nenhum menu do navegador ("Salvar imagem como...") em lugar nenhum, exceto nos campos de texto
document.addEventListener('contextmenu', (e) => {
  if (e.target.closest('input,textarea')) return;
  e.preventDefault();
  const el = e.target.closest('[data-i]');
  const it = el && items[+el.dataset.i];
  if (!it) return hideCtx();
  setSel(+el.dataset.i, false);
  openCtx(it, e.clientX, e.clientY);
});
ctxEl.addEventListener('click', async (e) => {
  const b = e.target.closest('button[data-n]');
  if (!b) return;
  const run = ctxActions[+b.dataset.n];
  hideCtx();
  await run?.();
});
ctxEl.addEventListener('keydown', (e) => {
  const btns = [...ctxEl.querySelectorAll('button')];
  const i = btns.indexOf(document.activeElement);
  if (e.key === 'ArrowDown') { e.preventDefault(); btns[(i + 1) % btns.length].focus(); }
  else if (e.key === 'ArrowUp') { e.preventDefault(); btns[(i - 1 + btns.length) % btns.length].focus(); }
  e.stopPropagation();
  if (e.key === 'Escape') { hideCtx(); input.focus(); }
});
document.addEventListener('mousedown', (e) => { if (!ctxEl.hidden && !e.target.closest('#ctx')) hideCtx(); });

// ───────────────────────── filtros / teclado ─────────────────────────
function setFilter(f) {
  filter = filter === f ? null : f;
  document.querySelectorAll('.cat').forEach((b) => b.classList.toggle('on', b.dataset.f === filter));
  confirmId = null;
  input.focus();
  refresh();
}
document.querySelectorAll('.cat').forEach((b) => b.addEventListener('click', () => setFilter(b.dataset.f)));

input.addEventListener('input', () => { clearBtn.hidden = !input.value; confirmId = null; refresh(); });
clearBtn.addEventListener('click', () => { input.value = ''; clearBtn.hidden = true; input.focus(); refresh(); });

pbody.addEventListener('click', async (e) => {
  const del = e.target.closest('[data-del]');
  if (del) { await invoke('clip_delete', { id: +del.dataset.del }); refresh(); return; }
  const row = e.target.closest('[data-i]');
  if (row) activate(items[+row.dataset.i], e.ctrlKey);
});
pbody.addEventListener('mouseover', (e) => { const row = e.target.closest('[data-i]'); if (row && +row.dataset.i !== sel) setSel(+row.dataset.i, false); });
phead.addEventListener('click', async (e) => { if (e.target.id === 'clipclear') { await invoke('clip_clear'); refresh(); } });
chipsEl.addEventListener('click', (e) => { const c = e.target.closest('.chip'); if (c) { fileKind = c.dataset.k; refresh(); } });
document.getElementById('stage').addEventListener('mousedown', (e) => { if (e.target.id === 'stage') hide(); });

document.addEventListener('keydown', async (e) => {
  const ctrl = e.ctrlKey || e.metaKey;

  if (e.key === 'Escape') {
    e.preventDefault();
    if (!ql.hidden) return closePreview();
    if (!qkbar.hidden) return closeQuickKey();
    if (!ctxEl.hidden) { hideCtx(); return input.focus(); }
    if (settingsOpen) return closeSettings();
    if (form) return exitForm();
    if (confirmId) { confirmId = null; return refreshRows(); }
    if (input.value || filter) {
      input.value = ''; clearBtn.hidden = true;
      filter = null; document.querySelectorAll('.cat').forEach((b) => b.classList.remove('on'));
      return refresh();
    }
    return hide();
  }
  if (form || settingsOpen || !qkbar.hidden) return;
  if (e.key === 'ContextMenu' || (e.shiftKey && e.key === 'F10')) {
    e.preventDefault();
    const el = pbody.querySelector('.sel');
    if (el && items[sel]) { const r = el.getBoundingClientRect(); openCtx(items[sel], r.left + 24, r.bottom - 8); }
    return;
  }
  if (ctrl && e.key === ',') { e.preventDefault(); return openSettings(); }

  if (ctrl && /^[1-4]$/.test(e.key)) { e.preventDefault(); return setFilter(['apps', 'files', 'actions', 'clip'][+e.key - 1]); }
  if (ctrl && e.key.toLowerCase() === 'k') { e.preventDefault(); return openQuickKey(); }
  if (ctrl && e.key.toLowerCase() === 'y') { e.preventDefault(); return openPreview(); }
  if (!ql.hidden) { if (e.key === ' ') { e.preventDefault(); closePreview(); } return; }
  if (e.key === ' ' && !input.value && items[sel] && ['file', 'folder'].includes(items[sel].kind)) { e.preventDefault(); return openPreview(); }

  const grid = cols > 1;
  if (e.key === 'ArrowDown') { e.preventDefault(); setSel(sel + cols); }
  else if (e.key === 'ArrowUp') { e.preventDefault(); setSel(sel - cols); }
  else if (e.key === 'ArrowRight' && (grid || !input.value)) { e.preventDefault(); setSel(sel + 1); }
  else if (e.key === 'ArrowLeft' && (grid || !input.value)) { e.preventDefault(); setSel(sel - 1); }
  else if (e.key === 'Tab') { e.preventDefault(); setSel(e.shiftKey ? sel - 1 : sel + 1); }
  else if (e.key === 'Enter') { e.preventDefault(); activate(items[sel], ctrl); }
  else if (e.key === 'Delete' && filter === 'apps' && !input.value && usage[items[sel]?.id] > 0) { e.preventDefault(); const it = items[sel]; await invoke('suggest_hide', { id: it.id, title: it.title }); neverSuggest.add(it.id); delete usage[it.id]; toast(`${it.title} nunca mais aparece nas sugestões (reative em Configurações)`); refresh(); }
  else if (e.key === 'Delete' && filter === 'clip' && items[sel]) { e.preventDefault(); await invoke('clip_delete', { id: items[sel].clipId }); refresh(); }
});

// ───────────────────────── ciclo de vida da janela ─────────────────────────
function applyTheme(p) {
  document.documentElement.dataset.theme = p.dark ? 'dark' : 'light';
  document.documentElement.style.setProperty('--accent', p.accent || '#0a84ff');
}

async function onShown(p) {
  applyTheme(p);
  winW = p.w; winH = p.h;
  setBackdrop(p.bd);
  input.value = ''; clearBtn.hidden = true; input.hidden = false;
  form = null; formEl.hidden = true; formEl.innerHTML = '';
  settingsOpen = false;
  filter = null; confirmId = null; ql.hidden = true; qkbar.hidden = true;
  document.querySelectorAll('.cat').forEach((b) => b.classList.remove('on'));
  showPanel(false);
  lastWinH = 0; fitWindow();
  const app = $('app');
  app.classList.remove('in'); void app.offsetWidth; app.classList.add('in');
  input.focus();
  let meta;
  [usage, quick, meta] = await Promise.all([invoke('usage_get'), invoke('quick_get'), invoke('config_get')]);
  neverSuggest = new Set(meta.config.never_suggest.map((h) => h.id));
  invoke('ssh_hosts').then((h) => { sshHosts = h; }).catch(() => {});
  pinnedIds = new Set(meta.config.pinned.map((h) => h.id));
  webIcons = meta.config.web_icons !== false;
  checkUpdates = meta.config.check_updates !== false;
  maybeCheckUpdate();
  warmRates();
}
function onHidden() { $('app').classList.remove('in'); }

T.event.listen('shown', (e) => onShown(e.payload));
T.event.listen('hidden', onHidden);
T.event.listen('open-settings', () => openSettings());
window.addEventListener('blur', () => {});
window.__spot = { activate, setFilter, get items() { return items; } }; // depuração
