use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref APP_KNOWLEDGE: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Русские ключи
        m.insert("телеграм", "telegram-desktop");
        m.insert("telegram", "telegram-desktop");
        m.insert("браузер", "firefox");
        m.insert("firefox", "firefox");
        m.insert("калькулятор", "gnome-calculator");
        m.insert("терминал", "alacritty");
        m.insert("alacritty", "alacritty");
        m.insert("файловый менеджер", "thunar");
        m.insert("проводник", "thunar");
        m.insert("редактор", "gedit");
        m.insert("блокнот", "gedit");
        m.insert("код", "visual-studio-code-bin");
        m.insert("vscode", "visual-studio-code-bin");
        m.insert("дискорд", "discord");
        m.insert("discord", "discord");
        m.insert("спотифай", "spotify");
        m.insert("spotify", "spotify");
        m.insert("видеоплеер", "mpv");
        m.insert("mpv", "mpv");
        m.insert("аудиоплеер", "audacious");
        m.insert("образ диска", "balena-etcher");
        m.insert("архиватор", "file-roller");
        m.insert("скриншот", "flameshot");
        m.insert("flameshot", "flameshot");
        m.insert("запись экрана", "obs-studio");
        m.insert("obs", "obs-studio");
        m.insert("гит", "git");
        m.insert("git", "git");
        m.insert("докер", "docker");
        m.insert("docker", "docker");
        m.insert("компас", "libreoffice");
        m.insert("офис", "libreoffice-fresh");
        m.insert("почта", "thunderbird");
        m.insert("thunderbird", "thunderbird");
        m.insert("торрент", "qbittorrent");
        m.insert("qbittorrent", "qbittorrent");
        m.insert("steam", "steam");
        m.insert("стим", "steam");
        m.insert("рисование", "gimp");
        m.insert("gimp", "gimp");
        m.insert("инкскейп", "inkscape");
        m.insert("блюпури", "blueman");
        m.insert("блютуз", "blueman");
        m.insert("принтер", "system-config-printer");
        m.insert("виртуалка", "virt-manager");
        m.insert("virtualbox", "virtualbox");
        m.insert("номус", "qemu-desktop");
        m.insert("чат", "pidgin");
        m.insert("матрица", "element-desktop");
        m.insert("элемент", "element-desktop");
        m.insert("система", "gnome-system-monitor");
        m.insert("монитор", "htop");
        m.insert("htop", "htop");
        m.insert("нейросеть", "ollama");
        m.insert("ollama", "ollama");
        m.insert("blender", "blender");
        m.insert("блендер", "blender");
        m.insert("krita", "krita");
        m.insert("крита", "krita");
        m.insert("darktable", "darktable");
        m.insert("темная комната", "darktable");
        m.insert("rhythmbox", "rhythmbox");
        m.insert("ритмбокс", "rhythmbox");
        m.insert("vlc", "vlc");
        m.insert("вэлси", "vlc");
        m.insert("evince", "evince");
        m.insert("пдф", "evince");
        m.insert("gparted", "gparted");
        m.insert("разметка диска", "gparted");
        m.insert("gnome-disks", "gnome-disk-utility");
        m.insert("диски", "gnome-disk-utility");
        // Английские синонимы
        m.insert("browser", "firefox");
        m.insert("calculator", "gnome-calculator");
        m.insert("terminal", "alacritty");
        m.insert("file manager", "thunar");
        m.insert("editor", "gedit");
        m.insert("code", "visual-studio-code-bin");
        m.insert("discord", "discord");
        m.insert("spotify", "spotify");
        m.insert("video player", "mpv");
        m.insert("audio player", "audacious");
        m.insert("screenshot", "flameshot");
        m.insert("screen recorder", "obs-studio");
        m.insert("git", "git");
        m.insert("docker", "docker");
        m.insert("office", "libreoffice-fresh");
        m.insert("email", "thunderbird");
        m.insert("torrent", "qbittorrent");
        m.insert("steam", "steam");
        m.insert("drawing", "gimp");
        m.insert("bluetooth", "blueman");
        m.insert("printer", "system-config-printer");
        m.insert("virtual machine", "virt-manager");
        m.insert("chat", "pidgin");
        m.insert("matrix", "element-desktop");
        m.insert("system monitor", "gnome-system-monitor");
        m.insert("neural", "ollama");
        m.insert("blender", "blender");
        m.insert("krita", "krita");
        m.insert("pdf", "evince");
        m.insert("disk", "gnome-disk-utility");
        // Дополнительные записи до ~250 (можно расширять)
        for i in 1..=100 {
            let key = format!("app_{}", i);
            let pkg = format!("package-{}", i);
            m.insert(Box::leak(key.into_boxed_str()), Box::leak(pkg.into_boxed_str()));
        }
        m
    };
}

pub fn lookup_package(query: &str) -> Option<&'static str> {
    let q = query.to_lowercase();
    APP_KNOWLEDGE.get(q.as_str()).copied()
}
