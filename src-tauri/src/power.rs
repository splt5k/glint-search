//! Indexação educada: baixa prioridade de CPU/disco e pausa com o notebook na bateria.

/// Rebaixa a thread atual (CPU e E/S) para não atrapalhar o uso do computador.
pub fn background_thread() {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::System::Threading::{GetCurrentThread, SetThreadPriority, THREAD_MODE_BACKGROUND_BEGIN};
        let _ = SetThreadPriority(GetCurrentThread(), THREAD_MODE_BACKGROUND_BEGIN);
    }
    #[cfg(target_os = "linux")]
    unsafe {
        // nice 19 só desta thread (no Linux a prioridade é por tid)
        let tid = libc::syscall(libc::SYS_gettid) as libc::id_t;
        libc::setpriority(libc::PRIO_PROCESS, tid, 19);
    }
}

pub fn on_battery() -> bool {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
        let mut s = SYSTEM_POWER_STATUS::default();
        // ACLineStatus: 0 = bateria, 1 = tomada, 255 = desconhecido
        GetSystemPowerStatus(&mut s).is_ok() && s.ACLineStatus == 0
    }
    #[cfg(target_os = "linux")]
    {
        std::fs::read_dir("/sys/class/power_supply")
            .map(|rd| {
                rd.flatten().any(|e| {
                    std::fs::read_to_string(e.path().join("status")).map(|s| s.trim() == "Discharging").unwrap_or(false)
                })
            })
            .unwrap_or(false)
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        false
    }
}

/// Wayland não deixa aplicativos registrarem atalhos globais; a UI avisa e sugere alternativa.
pub fn wayland() -> bool {
    cfg!(target_os = "linux") && std::env::var("XDG_SESSION_TYPE").map(|v| v == "wayland").unwrap_or(false)
}
