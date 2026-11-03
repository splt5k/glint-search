use crate::fuzzy;
use crate::index::Hit;

/// (título, uri, palavras-chave)
#[cfg(windows)]
const PAGES: &[(&str, &str, &[&str])] = &[
    ("Wi-Fi", "ms-settings:network-wifi", &["wifi", "wi-fi", "internet", "rede sem fio"]),
    ("Rede e Internet", "ms-settings:network", &["rede", "network", "ethernet", "vpn", "proxy"]),
    ("Bluetooth e dispositivos", "ms-settings:bluetooth", &["bluetooth", "dispositivos", "mouse", "impressora"]),
    ("Tela", "ms-settings:display", &["tela", "display", "monitor", "resolução", "brilho", "escala", "hdr"]),
    ("Som", "ms-settings:sound", &["som", "sound", "áudio", "volume", "microfone", "alto-falante"]),
    ("Notificações", "ms-settings:notifications", &["notificações", "notifications", "não perturbe"]),
    ("Energia e bateria", "ms-settings:powersleep", &["energia", "bateria", "power", "suspensão"]),
    ("Armazenamento", "ms-settings:storagesense", &["armazenamento", "storage", "disco", "espaço"]),
    ("Aplicativos instalados", "ms-settings:appsfeatures", &["aplicativos", "apps", "desinstalar", "programas"]),
    ("Aplicativos padrão", "ms-settings:defaultapps", &["padrão", "default", "navegador padrão"]),
    ("Aplicativos de inicialização", "ms-settings:startupapps", &["inicialização", "startup", "iniciar com o windows"]),
    ("Personalização", "ms-settings:personalization", &["personalização", "tema", "papel de parede", "fundo"]),
    ("Modo escuro e cores", "ms-settings:colors", &["modo escuro", "dark mode", "cores", "cor de destaque"]),
    ("Barra de tarefas", "ms-settings:taskbar", &["barra de tarefas", "taskbar"]),
    ("Windows Update", "ms-settings:windowsupdate", &["update", "atualização", "atualizar"]),
    ("Privacidade e segurança", "ms-settings:privacy", &["privacidade", "privacy", "segurança", "localização"]),
    ("Segurança do Windows", "windowsdefender:", &["antivírus", "defender", "vírus", "firewall"]),
    ("Contas", "ms-settings:yourinfo", &["conta", "contas", "perfil", "senha", "login"]),
    ("Hora e idioma", "ms-settings:dateandtime", &["hora", "data", "fuso", "idioma", "language", "região"]),
    ("Teclado", "ms-settings:typing", &["teclado", "keyboard", "digitação", "autocorreção"]),
    ("Acessibilidade", "ms-settings:easeofaccess", &["acessibilidade", "narrador", "lupa", "legendas"]),
    ("Sobre este computador", "ms-settings:about", &["sobre", "about", "especificações", "versão do windows", "nome do pc"]),
    ("Jogos", "ms-settings:gaming-gamebar", &["jogos", "game bar", "modo jogo"]),
    ("Impressoras e scanners", "ms-settings:printers", &["impressora", "scanner", "printer"]),
    ("Mouse", "ms-settings:mousetouchpad", &["mouse", "touchpad", "ponteiro"]),
];

#[cfg(not(windows))]
const PAGES: &[(&str, &str, &[&str])] = &[];

pub fn search(query: &str, list_all: bool) -> Vec<Hit> {
    let q = query.trim().to_lowercase();
    let mut out = Vec::new();
    for (title, uri, kws) in PAGES {
        let mut s = if q.is_empty() {
            if list_all { 300 } else { 0 }
        } else {
            fuzzy::score_lower(&q, &title.to_lowercase())
        };
        if !q.is_empty() {
            for k in *kws {
                s = s.max(fuzzy::score_lower(&q, &k.to_lowercase()) - 50);
            }
        }
        if s > 250 {
            out.push(Hit {
                id: format!("setting:{}", uri),
                kind: "setting".into(),
                title: title.to_string(),
                subtitle: "Configurações do Windows".into(),
                path: uri.to_string(),
                score: s,
                mtime: 0,
            });
        }
    }
    out
}
