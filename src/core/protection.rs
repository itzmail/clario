//! Path/app protection preventing `clean`/`purge`/`analyze` from deleting
//! critical system paths. The macOS section is ported from Mole's
//! `lib/core/app_protection.sh` and `lib/core/file_ops.sh` (system-critical
//! and data-sensitive app bundles, endpoint security agent caches). Linux gets
//! a smaller root-path-only guard — no app-bundle concept there.
//!
//! Only the clean/purge-relevant subset is ported. Mole's uninstall-specific
//! leftover-discovery heuristics (`find_app_files`, sibling-guard, vendor-nested
//! matching) are out of scope here — that logic belongs to `uninstall.rs`, not
//! this filter.

use std::path::Path;

/// Critical macOS system apps/components, protected from uninstall/cleanup.
/// Ported verbatim from Mole's `SYSTEM_CRITICAL_BUNDLES`.
#[cfg(target_os = "macos")]
const SYSTEM_CRITICAL_BUNDLES: &[&str] = &[
    // Core system applications (in /System/Applications/)
    "com.apple.finder",
    "com.apple.dock",
    "com.apple.Safari",
    "com.apple.mail",
    "com.apple.systempreferences",
    "com.apple.SystemSettings",
    "com.apple.Settings*",
    "com.apple.controlcenter*",
    "com.apple.Spotlight",
    "com.apple.notificationcenterui",
    "com.apple.loginwindow",
    "com.apple.Preview",
    "com.apple.TextEdit",
    "com.apple.Notes",
    "com.apple.reminders",
    "com.apple.iCal",
    "com.apple.AddressBook",
    "com.apple.Photos",
    "com.apple.AppStore",
    "com.apple.calculator",
    "com.apple.Dictionary",
    "com.apple.ScreenSharing",
    "com.apple.ActivityMonitor",
    "com.apple.Console",
    "com.apple.DiskUtility",
    "com.apple.KeychainAccess",
    "com.apple.DigitalColorMeter",
    "com.apple.grapher",
    "com.apple.Terminal",
    "com.apple.ScriptEditor2",
    "com.apple.VoiceOverUtility",
    "com.apple.BluetoothFileExchange",
    "com.apple.print.PrinterProxy",
    "com.apple.systempreferences*",
    "com.apple.SystemProfiler",
    "com.apple.FontBook",
    "com.apple.ColorSyncUtility",
    "com.apple.audio.AudioMIDISetup",
    "com.apple.DirectoryUtility",
    "com.apple.NetworkUtility",
    "com.apple.exposelauncher",
    "com.apple.MigrateAssistant",
    "com.apple.RAIDUtility",
    "com.apple.BootCampAssistant",
    // System services and daemons
    "com.apple.SecurityAgent",
    "com.apple.CoreServices*",
    "com.apple.SystemUIServer",
    "com.apple.backgroundtaskmanagement*",
    "com.apple.loginitems*",
    "com.apple.sharedfilelist*",
    "com.apple.sfl*",
    "com.apple.coreservices*",
    "com.apple.metadata*",
    "com.apple.MobileSoftwareUpdate*",
    "com.apple.SoftwareUpdate*",
    "com.apple.installer*",
    "com.apple.frameworks*",
    "com.apple.security*",
    "com.apple.keychain*",
    "com.apple.trustd*",
    "com.apple.securityd*",
    "com.apple.cloudd*",
    "com.apple.iCloud*",
    "com.apple.WiFi*",
    "com.apple.airport*",
    "com.apple.Bluetooth*",
    // Input methods (system built-in)
    "com.apple.inputmethod.*",
    "com.apple.inputsource*",
    "com.apple.TextInput*",
    "com.apple.CharacterPicker*",
    "com.apple.PressAndHold*",
    // Legacy pattern-based entries (non com.apple.*)
    "loginwindow",
    "dock",
    "systempreferences",
    "finder",
    "safari",
    "backgroundtaskmanagementagent",
    "keychain*",
    "security*",
    "bluetooth*",
    "wifi*",
    "network*",
    "tcc",
    "notification*",
    "accessibility*",
    "universalaccess*",
    "HIToolbox*",
    "textinput*",
    "TextInput*",
    "keyboard*",
    "Keyboard*",
    "inputsource*",
    "InputSource*",
    "keylayout*",
    "KeyLayout*",
    "GlobalPreferences",
    ".GlobalPreferences",
];

/// Third-party apps with sensitive data (credentials, licenses, project state),
/// protected from cache cleanup even though their cache dirs look regenerable.
/// Ported verbatim from Mole's `DATA_PROTECTED_BUNDLES`.
#[cfg(target_os = "macos")]
const DATA_PROTECTED_BUNDLES: &[&str] = &[
    // Input Methods
    "com.tencent.inputmethod.QQInput",
    "com.sogou.inputmethod.*",
    "com.baidu.inputmethod.*",
    "com.googlecode.rimeime.*",
    "im.rime.*",
    "*.inputmethod",
    "*.InputMethod",
    "*IME",
    // System Utilities & Cleanup
    "com.nektony.*",
    "com.macpaw.*",
    "com.freemacsoft.AppCleaner",
    "com.omnigroup.omnidisksweeper",
    "com.daisydiskapp.*",
    "com.tunabellysoftware.*",
    "com.grandperspectiv.*",
    "com.binaryfruit.*",
    // Password Managers
    "com.1password.*",
    "com.agilebits.*",
    "com.lastpass.*",
    "com.dashlane.*",
    "com.bitwarden.*",
    "com.keepassx.*",
    "org.keepassx.*",
    "org.keepassxc.*",
    "com.authy.*",
    "com.yubico.*",
    // IDEs & Editors
    "com.jetbrains.*",
    "JetBrains*",
    "com.microsoft.VSCode",
    "com.visualstudio.code.*",
    "com.sublimetext.*",
    "com.sublimehq.*",
    "com.microsoft.VSCodeInsiders",
    "com.apple.dt.Xcode",
    "com.coteditor.CotEditor",
    "com.macromates.TextMate",
    "com.panic.Nova",
    "abnerworks.Typora",
    "com.uranusjr.macdown",
    // AI & LLM Tools
    "com.todesktop.*",
    "Cursor",
    "com.anthropic.claude*",
    "Claude",
    "com.openai.chat*",
    "ChatGPT",
    "com.openai.codex",
    "Codex",
    "codex-runtimes",
    "com.ollama.ollama",
    "Ollama",
    "com.lmstudio.lmstudio",
    "LM Studio",
    "co.supertool.chatbox",
    "page.jan.jan",
    "com.huggingface.huggingchat",
    "Gemini",
    "com.perplexity.Perplexity",
    "com.drawthings.DrawThings",
    "com.divamgupta.diffusionbee",
    "com.exafunction.windsurf",
    "com.quora.poe.electron",
    "chat.openai.com.*",
    // Database Clients
    "com.sequelpro.*",
    "com.sequel-ace.*",
    "com.tinyapp.*",
    "com.dbeaver.*",
    "com.navicat.*",
    "com.mongodb.compass",
    "com.redis.RedisInsight",
    "com.pgadmin.pgadmin4",
    "com.eggerapps.Sequel-Pro",
    "com.valentina-db.Valentina-Studio",
    "com.dbvis.DbVisualizer",
    // API & Network Tools
    "com.postmanlabs.mac",
    "com.konghq.insomnia",
    "com.CharlesProxy.*",
    "com.proxyman.*",
    "com.getpaw.*",
    "com.luckymarmot.Paw",
    "com.charlesproxy.charles",
    "com.telerik.Fiddler",
    "com.usebruno.app",
    // Network Proxy & VPN Tools
    "com.clash.*",
    "ClashX*",
    "clash-*",
    "Clash-*",
    "*-clash",
    "*-Clash",
    "clash.*",
    "Clash.*",
    "clash_*",
    "*clash-verge*",
    "*Clash-Verge*",
    "clashverge*",
    "ClashVerge*",
    "com.nssurge.surge-mac",
    "*surge*",
    "*Surge*",
    "mihomo*",
    "*openvpn*",
    "*OpenVPN*",
    "net.openvpn.*",
    // Proxy Clients
    "*ShadowsocksX-NG*",
    "com.qiuyuzhou.*",
    "*v2ray*",
    "*V2Ray*",
    "*v2box*",
    "*V2Box*",
    "*nekoray*",
    "*sing-box*",
    "*OneBox*",
    "*hiddify*",
    "*Hiddify*",
    "*loon*",
    "*Loon*",
    "*quantumult*",
    // Mesh & Corporate VPNs
    "*tailscale*",
    "io.tailscale.*",
    "*zerotier*",
    "com.zerotier.*",
    "*1dot1dot1dot1*",
    "*cloudflare*warp*",
    "org.amnezia.*",
    "*amnezia*",
    "*Amnezia*",
    "com.wireguard.*",
    "*wireguard*",
    "*WireGuard*",
    // Commercial VPNs
    "*nordvpn*",
    "*expressvpn*",
    "*protonvpn*",
    "*surfshark*",
    "*windscribe*",
    "*mullvad*",
    "*privateinternetaccess*",
    // Screensaver & Wallpaper
    "*Aerial.saver*",
    "com.JohnCoates.Aerial*",
    "*Fliqlo*",
    "*fliqlo*",
    // Git & Version Control
    "com.github.GitHubDesktop",
    "com.sublimemerge",
    "com.torusknot.SourceTreeNotMAS",
    "com.git-tower.Tower*",
    "com.gitfox.GitFox",
    "com.github.Gitify",
    "com.fork.Fork",
    "com.axosoft.gitkraken",
    // Terminal & Shell
    "com.googlecode.iterm2",
    "net.kovidgoyal.kitty",
    "io.alacritty",
    "com.github.wez.wezterm",
    "com.hyper.Hyper",
    "com.mizage.divvy",
    "com.fig.Fig",
    "dev.warp.Warp-Stable",
    "com.termius-dmg",
    // Docker & Virtualization
    "com.docker.docker",
    "dev.orbstack.OrbStack",
    "dev.orbstack.*",
    "dev.kdrag0n.MacVirt",
    "com.getutm.UTM",
    "com.vmware.fusion",
    "com.parallels.desktop.*",
    "org.virtualbox.app.VirtualBox",
    "com.vagrant.*",
    "com.orbstack.OrbStack",
    // System Monitoring
    "com.bjango.istatmenus*",
    "eu.exelban.Stats",
    "com.monitorcontrol.*",
    "com.bresink.system-toolkit.*",
    "com.mediaatelier.MenuMeters",
    "com.activity-indicator.app",
    "net.cindori.sensei",
    // Window Management
    "com.macitbetter.*",
    "com.hegenberg.*",
    "com.manytricks.*",
    "com.divisiblebyzero.*",
    "com.koingdev.*",
    "com.if.Amphetamine",
    "com.lwouis.alt-tab-macos",
    "net.matthewpalmer.Vanilla",
    "com.lightheadsw.Caffeine",
    "com.contextual.Contexts",
    "com.amethyst.Amethyst",
    "com.knollsoft.Rectangle",
    "com.knollsoft.Hookshot",
    "com.surteesstudios.Bartender",
    "com.gaosun.eul",
    "com.pointum.hazeover",
    // Launcher & Automation
    "com.runningwithcrayons.Alfred",
    "com.raycast.*",
    "com.raycast-x.*",
    "com.blacktree.Quicksilver",
    "com.stairways.keyboardmaestro.*",
    "com.manytricks.Butler",
    "com.happenapps.Quitter",
    "com.pilotmoon.scroll-reverser",
    "org.pqrs.Karabiner-Elements",
    "com.apple.Automator",
    // Note-Taking
    "com.bear-writer.*",
    "com.typora.*",
    "com.ulyssesapp.*",
    "com.literatureandlatte.*",
    "com.dayoneapp.*",
    "notion.id",
    "md.obsidian",
    "com.logseq.logseq",
    "com.evernote.Evernote",
    "com.onenote.mac",
    "com.omnigroup.OmniOutliner*",
    "net.shinyfrog.bear",
    "com.goodnotes.GoodNotes",
    "com.marginnote.MarginNote*",
    "com.roamresearch.*",
    "com.reflect.ReflectApp",
    "com.inkdrop.*",
    // Design & Creative
    "com.adobe.*",
    "com.avid.mediacomposer*",
    "com.bohemiancoding.*",
    "com.figma.*",
    "com.framerx.*",
    "com.zeplin.*",
    "com.invisionapp.*",
    "com.principle.*",
    "com.pixelmatorteam.*",
    "com.affinitydesigner.*",
    "com.affinityphoto.*",
    "com.affinitypublisher.*",
    "com.linearity.curve",
    "com.canva.CanvaDesktop",
    "com.maxon.cinema4d",
    "com.autodesk.*",
    "com.sketchup.*",
    "com.native-instruments.*",
    "com.fabfilter.*",
    "com.paceap.*",
    "com.izotope.*",
    "iZotope",
    "com.lasersoft-imaging.*",
    "app.cotypist.Cotypist",
    // Communication
    "com.tencent.xinWeChat",
    "com.tencent.qq",
    "com.alibaba.DingTalkMac",
    "com.alibaba.AliLang.osx",
    "com.alibaba.alilang3.osx.ShipIt",
    "com.alibaba.AlilangMgr.QueryNetworkInfo",
    "us.zoom.xos",
    "com.microsoft.teams*",
    "com.slack.Slack",
    "com.hnc.Discord",
    "app.legcord.Legcord",
    "org.telegram.desktop",
    "ru.keepcoder.Telegram",
    "net.whatsapp.WhatsApp",
    "com.skype.skype",
    "com.cisco.webexmeetings",
    "com.ringcentral.RingCentral",
    "com.readdle.smartemail-Mac",
    "com.airmail.*",
    "com.postbox-inc.postbox",
    "com.tinyspeck.slackmacgap",
    // Task Management
    "com.omnigroup.OmniFocus*",
    "com.culturedcode.*",
    "com.todoist.*",
    "com.any.do.*",
    "com.ticktick.*",
    "com.microsoft.to-do",
    "com.trello.trello",
    "com.asana.nativeapp",
    "com.clickup.*",
    "com.monday.desktop",
    "com.airtable.airtable",
    "com.notion.id",
    "com.linear.linear",
    // File Transfer & Sync
    "com.panic.transmit*",
    "com.binarynights.ForkLift*",
    "com.noodlesoft.Hazel",
    "com.cyberduck.Cyberduck",
    "io.filezilla.FileZilla",
    "com.apple.Xcode.CloudDocuments",
    "com.synology.*",
    // Cloud Storage & Backup
    "com.dropbox.*",
    "com.getdropbox.*",
    "*dropbox*",
    "ws.agile.*",
    "com.backblaze.*",
    "*backblaze*",
    "com.box.desktop*",
    "*box.desktop*",
    "com.microsoft.OneDrive*",
    "com.microsoft.SyncReporter",
    "*OneDrive*",
    "com.google.GoogleDrive",
    "com.google.keystone*",
    "*GoogleDrive*",
    "com.amazon.drive",
    "com.apple.bird",
    "com.apple.CloudDocs*",
    "com.displaylink.*",
    "com.fujitsu.pfu.ScanSnap*",
    "com.citrix.*",
    "org.xquartz.*",
    "us.zoom.updater*",
    "com.DigiDNA.iMazing*",
    "com.shirtpocket.*",
    "homebrew.mxcl.*",
    // Remote Desktop / Remote Access
    "org.chromium.chromoting*",
    "com.google.chrome_remote_desktop*",
    "com.teamviewer.*",
    "com.realvnc.*",
    "com.logmein.*",
    "com.anydesk.*",
    // Screenshot & Recording
    "com.cleanshot.*",
    "com.xnipapp.xnip",
    "com.reincubate.camo",
    "com.tunabellysoftware.ScreenFloat",
    "net.telestream.screenflow*",
    "com.techsmith.snagit*",
    "com.techsmith.camtasia*",
    "com.obsidianapp.screenrecorder",
    "com.kap.Kap",
    "com.getkap.*",
    "com.linebreak.CloudApp",
    "com.droplr.droplr-mac",
    // Media & Entertainment
    "com.spotify.client",
    "com.apple.Music",
    "com.apple.podcasts",
    "com.apple.BKAgentService",
    "com.apple.iBooksX",
    "com.apple.iBooks",
    "com.blackmagic-design.*",
    "com.colliderli.iina",
    "org.videolan.vlc",
    "io.mpv",
    "tv.plex.player.desktop",
    "com.netease.163music",
    // Web Browsers
    "Firefox",
    "org.mozilla.*",
    // Scientific & Professional Software
    "com.crowdstrike.*",
    "com.kolide.*",
    "com.sas.*",
    "com.mathworks.*",
    "com.ibm.spss.*",
    "com.wolfram.*",
    "com.stata.*",
    "org.rstudio.*",
    "com.tableausoftware.*",
    // License & App Stores
    "com.paddle.Paddle*",
    "com.quicken.*",
    "com.setapp.DesktopClient",
    "com.devmate.*",
    "org.sparkle-project.Sparkle*",
];

/// Endpoint security / EDR / MDM agent bundle-id prefixes. Their caches live in
/// ordinary-looking `/private/var/folders/*` paths, but deleting anything inside
/// trips sensor tamper detection (reported as malware by corporate security).
/// Ported verbatim from Mole's `ENDPOINT_SECURITY_BUNDLE_PREFIXES`.
#[cfg(target_os = "macos")]
const ENDPOINT_SECURITY_BUNDLE_PREFIXES: &[&str] = &[
    "com.crowdstrike.",
    "com.sentinelone.",
    "com.sentinel-labs.",
    "com.eset.",
    "com.jamf.",
    "com.jamfsoftware.",
    "com.paloaltonetworks.",
    "com.cisco.anyconnect",
    "com.cisco.secureclient",
];

/// Case-insensitive glob match supporting only `*` (Mole's bundle patterns never
/// use `?`/`[...]`). `*` matches any run of characters, including empty.
fn glob_match(pattern: &str, value: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let value = value.to_lowercase();
    glob_match_lower(&pattern, &value)
}

fn glob_match_lower(pattern: &str, value: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == value,
        Some((prefix, rest)) => {
            let Some(remaining) = value.strip_prefix(prefix) else { return false };
            if rest.is_empty() {
                return true;
            }
            // Try every split point after the prefix, matching the rest of the
            // pattern against increasingly shorter suffixes (handles multiple `*`).
            for i in 0..=remaining.len() {
                if remaining.is_char_boundary(i) && glob_match_lower(rest, &remaining[i..]) {
                    return true;
                }
            }
            false
        }
    }
}

#[cfg(target_os = "macos")]
fn matches_any(patterns: &[&str], token: &str) -> bool {
    patterns.iter().any(|p| glob_match(p, token))
}

/// Mirrors Mole's `should_protect_data()`: does this bundle ID (or app/file
/// name) belong to a system-critical or data-sensitive app?
/// macOS only — Linux has no app-bundle concept to check against.
#[cfg(target_os = "macos")]
pub fn should_protect_data(token: &str) -> bool {
    matches_any(SYSTEM_CRITICAL_BUNDLES, token) || matches_any(DATA_PROTECTED_BUNDLES, token)
}

/// Mirrors Mole's `is_endpoint_security_cache_path()`: EDR agent caches under
/// `var/folders` look rebuildable but must never be touched. macOS only —
/// `/var/folders` doesn't exist as an EDR cache convention on Linux.
pub fn is_endpoint_security_cache_path(path: &Path) -> bool {
    #[cfg(target_os = "macos")]
    {
        let path_str = path.to_string_lossy();
        let in_var_folders = path_str.starts_with("/private/var/folders/") || path_str.starts_with("/var/folders/");
        if !in_var_folders {
            return false;
        }
        ENDPOINT_SECURITY_BUNDLE_PREFIXES.iter().any(|prefix| path_str.contains(prefix))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        false
    }
}

/// Hardcoded critical system directories that must never be deleted, regardless
/// of category. macOS list ported from Mole's `_mole_is_critical_deletion_path()`.
/// Linux list is a smaller root-path-only guard (no app-bundle equivalent).
fn is_critical_deletion_path(path: &Path) -> bool {
    let s = path.to_string_lossy();

    #[cfg(target_os = "macos")]
    {
        // Homebrew and user-installed software live here; individual entries stay
        // deletable, but the roots themselves fall through to the checks below.
        if s.starts_with("/usr/local/") || s.starts_with("/opt/homebrew/") {
            return false;
        }

        const EXACT_OR_PREFIX_ROOTS: &[&str] = &[
            "/bin", "/sbin", "/usr", "/System", "/Library/Apple", "/Applications/Finder.app",
            "/Applications/Safari.app", "/etc", "/private/etc", "/var/db", "/private/var/db",
            "/var/audit", "/private/var/audit",
        ];
        for root in EXACT_OR_PREFIX_ROOTS {
            if s == *root || s.starts_with(&format!("{root}/")) {
                return true;
            }
        }

        const EXACT_ROOTS: &[&str] = &[
            "/",
            "/Library",
            "/Library/Application Support",
            "/Library/Extensions",
            "/Library/Keychains",
            "/Applications",
            "/Volumes",
            "/opt",
            "/opt/homebrew",
            "/Users",
            "/Users/Shared",
            "/Users/Guest",
            "/private",
            "/var",
            "/var/db",
            "/var/root",
            "/private/var",
            "/private/var/root",
        ];
        if EXACT_ROOTS.contains(&s.as_ref()) {
            return true;
        }
        if s.starts_with("/Library/Extensions/")
            || s.starts_with("/Library/Keychains/")
            || s.starts_with("/Users/Guest/")
        {
            return true;
        }

        // Reject a user home root (/Users/<name>) while keeping its children deletable.
        if let Some(rest) = s.strip_prefix("/Users/") {
            if !rest.is_empty() && !rest.contains('/') {
                return true;
            }
        }

        return false;
    }

    #[cfg(target_os = "linux")]
    {
        const EXACT_OR_PREFIX_ROOTS: &[&str] =
            &["/bin", "/sbin", "/usr", "/etc", "/boot", "/proc", "/sys", "/dev", "/lib", "/lib64", "/root"];
        for root in EXACT_OR_PREFIX_ROOTS {
            if s == *root || s.starts_with(&format!("{root}/")) {
                return true;
            }
        }

        const EXACT_ROOTS: &[&str] = &["/", "/var", "/opt", "/home", "/mnt", "/media"];
        if EXACT_ROOTS.contains(&s.as_ref()) {
            return true;
        }

        // Reject a user home root (/home/<name>) while keeping its children deletable.
        if let Some(rest) = s.strip_prefix("/home/") {
            if !rest.is_empty() && !rest.contains('/') {
                return true;
            }
        }

        return false;
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = s;
        false
    }
}

/// Extract the bundle ID from a sandboxed container path, e.g.
/// `.../Library/Containers/<bundle-id>/...` or `.../Library/Group Containers/<bundle-id>/...`.
#[cfg(target_os = "macos")]
fn extract_container_bundle_id(path_str: &str) -> Option<&str> {
    for marker in ["/Library/Containers/", "/Library/Group Containers/"] {
        if let Some(idx) = path_str.find(marker) {
            let rest = &path_str[idx + marker.len()..];
            return rest.split('/').next().filter(|s| !s.is_empty());
        }
    }
    None
}

/// Mirrors Mole's `should_protect_path()`: is this path protected from
/// deletion regardless of which scan category found it?
///
/// Ported subset: system UI keyword matches, sandboxed container bundle-ID
/// checks, endpoint-security agent caches, critical preference files, iCloud/
/// Keychain/Mail/Contacts/Calendars, audio subsystem caches, and the full
/// bundle-pattern sweep. Skipped: uninstall-mode-only branches (this is a
/// clean/purge filter, not an uninstall leftover finder).
/// macOS only — none of these app-bundle/plist conventions apply on Linux;
/// `is_critical_deletion_path`'s root-path guard is the Linux equivalent.
#[cfg(not(target_os = "macos"))]
pub fn should_protect_path(_path: &Path) -> bool {
    false
}

#[cfg(target_os = "macos")]
pub fn should_protect_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let lower = path_str.to_lowercase();

    // 1. System UI keyword matches (case-insensitive).
    for needle in ["systemsettings", "systempreferences", "controlcenter", "com.apple.settings", "com.apple.notes"] {
        if lower.contains(needle) {
            return true;
        }
    }

    // 2. Caches critical for system UI rendering.
    for needle in [
        "com.apple.systempreferences.cache",
        "com.apple.settings.cache",
        "com.apple.controlcenter.cache",
        "com.apple.finder.cache",
        "com.apple.dock.cache",
        "/library/containers/com.apple.settings",
        "/library/containers/com.apple.systemsettings",
        "/library/containers/com.apple.controlcenter",
        "/library/group containers/com.apple.systempreferences",
        "/library/group containers/com.apple.settings",
        "/orbstack",
    ] {
        if lower.contains(needle) {
            return true;
        }
    }

    // 3. Sandboxed container bundle-ID extraction.
    if let Some(bundle_id) = extract_container_bundle_id(&path_str) {
        let is_container_cache = lower.contains("/data/library/caches/") || lower.contains("/data/tmp/");
        if !is_container_cache && should_protect_data(bundle_id) {
            return true;
        }
    }

    // 4. Specific hardcoded critical patterns.
    for needle in ["com.apple.settings", "com.apple.systemsettings", "com.apple.controlcenter", "com.apple.finder", "com.apple.dock"] {
        if lower.contains(needle) {
            return true;
        }
    }

    // 4b. Endpoint security / EDR agent caches.
    if is_endpoint_security_cache_path(path) {
        return true;
    }

    // 5. Critical preference files and user data.
    if path_str.ends_with("/Library/Preferences/com.apple.dock.plist")
        || path_str.ends_with("/Library/Preferences/com.apple.finder.plist")
        || lower.contains("/library/logs/mole")
        || lower.contains("/library/preferences/com.apple.networkextension")
        || lower.contains("mobile documents")
    {
        return true;
    }
    for needle in [
        "/byhost/com.apple.bluetooth.",
        "/byhost/com.apple.wifi.",
        "/library/accounts",
        "/library/keychains",
        "/library/mail",
        "/library/calendars",
        "/library/contacts",
    ] {
        if lower.contains(needle) {
            return true;
        }
    }

    // 6. Audio subsystem caches (issue #553): cleaning these can cause audio
    // output loss on Intel Macs.
    if lower.contains("com.apple.coreaudio") || lower.contains("com.apple.audio.") || lower.contains("coreaudiod") {
        return true;
    }

    // 7. Full bundle-pattern sweep against the whole path (catches e.g.
    // `~/Library/Caches/Claude` when the pattern is `Claude`).
    if matches_any(SYSTEM_CRITICAL_BUNDLES, &path_str) || matches_any(DATA_PROTECTED_BUNDLES, &path_str) {
        return true;
    }

    // 8. Filename itself against data-protected patterns.
    if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
        if matches_any(DATA_PROTECTED_BUNDLES, filename) {
            return true;
        }
    }

    false
}

/// Mirrors Mole's `validate_path_for_deletion()`: the combined critical-root +
/// protected-path check every deletion should pass before proceeding.
/// Returns `true` when the path is safe to delete.
pub fn is_safe_to_delete(path: &Path) -> bool {
    if is_critical_deletion_path(path) {
        return false;
    }
    if is_endpoint_security_cache_path(path) {
        return false;
    }
    if should_protect_path(path) {
        return false;
    }
    true
}

/// User-defined exceptions loaded from `~/.config/clario/whitelist`, one
/// pattern per line. `#`-prefixed lines are comments, `~` expands to $HOME.
/// Supports `*` glob plus parent/child directory containment, mirroring
/// Mole's `is_path_whitelisted()`.
pub fn load_whitelist() -> Vec<String> {
    let Some(config) = dirs::config_dir() else { return vec![] };
    let path = config.join("clario/whitelist");
    let Ok(content) = std::fs::read_to_string(&path) else { return vec![] };
    let Some(home) = dirs::home_dir() else { return vec![] };
    let home_str = home.to_string_lossy();

    content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            if let Some(rest) = l.strip_prefix('~') {
                format!("{home_str}{rest}")
            } else {
                l.to_string()
            }
        })
        .collect()
}

/// Is `path` covered by a whitelist pattern — either an exact/glob match, an
/// ancestor of a whitelisted path, or a descendant of one?
pub fn is_path_whitelisted(path: &Path, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }
    let target = path.to_string_lossy();
    let target = target.trim_end_matches('/');

    for pattern in patterns {
        let check = pattern.trim_end_matches('/');
        let has_glob = check.contains('*');

        if (has_glob && glob_match(check, target)) || (!has_glob && check == target) {
            return true;
        }
        // target is an ancestor of a whitelisted path.
        if check.starts_with(&format!("{target}/")) {
            return true;
        }
        // target is a descendant of a whitelisted (non-glob) path.
        if !has_glob && target.starts_with(&format!("{check}/")) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn glob_matches_prefix_and_suffix() {
        assert!(glob_match("com.apple.Settings*", "com.apple.Settings.cache"));
        assert!(glob_match("*surge*", "com.example.Surge.Helper"));
        assert!(!glob_match("com.apple.Settings*", "com.apple.NotSettings"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn protects_system_critical_bundle() {
        assert!(should_protect_data("com.apple.finder"));
        assert!(should_protect_data("com.apple.Settings.extra"));
        assert!(!should_protect_data("com.example.myrandomapp"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn protects_data_sensitive_bundle() {
        assert!(should_protect_data("com.1password.7"));
        assert!(should_protect_data("Claude"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn protects_endpoint_security_cache_path() {
        let p = PathBuf::from("/private/var/folders/ab/xyz/C/com.crowdstrike.falcon/cache");
        assert!(is_endpoint_security_cache_path(&p));
        let p2 = PathBuf::from("/private/var/folders/ab/xyz/C/com.example.app/cache");
        assert!(!is_endpoint_security_cache_path(&p2));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn protects_critical_roots() {
        assert!(is_critical_deletion_path(Path::new("/System")));
        assert!(is_critical_deletion_path(Path::new("/System/Library")));
        assert!(is_critical_deletion_path(Path::new("/Users/alice")));
        assert!(!is_critical_deletion_path(Path::new("/Users/alice/Documents")));
        assert!(!is_critical_deletion_path(Path::new("/usr/local/bin")));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn protects_critical_roots_linux() {
        assert!(is_critical_deletion_path(Path::new("/")));
        assert!(is_critical_deletion_path(Path::new("/etc")));
        assert!(is_critical_deletion_path(Path::new("/etc/passwd")));
        assert!(is_critical_deletion_path(Path::new("/usr/bin")));
        assert!(is_critical_deletion_path(Path::new("/home/alice")));
        assert!(!is_critical_deletion_path(Path::new("/home/alice/Documents")));
        assert!(!is_critical_deletion_path(Path::new("/home/alice/.cache")));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn protects_container_bundle_id() {
        let p = PathBuf::from("/Users/alice/Library/Containers/com.1password.7/Data/foo");
        assert!(should_protect_path(&p));
        // Cache/tmp inside a container is regenerable, let it through.
        let cache = PathBuf::from("/Users/alice/Library/Containers/com.1password.7/Data/Library/Caches/x");
        assert!(!should_protect_path(&cache));
    }

    #[test]
    fn whitelist_exact_and_containment() {
        let patterns = vec!["/Users/alice/keep".to_string()];
        assert!(is_path_whitelisted(Path::new("/Users/alice/keep"), &patterns));
        assert!(is_path_whitelisted(Path::new("/Users/alice/keep/sub"), &patterns));
        assert!(!is_path_whitelisted(Path::new("/Users/alice/other"), &patterns));
    }

    #[test]
    fn whitelist_glob() {
        let patterns = vec!["/Users/alice/Library/Caches/*".to_string()];
        assert!(is_path_whitelisted(Path::new("/Users/alice/Library/Caches/foo"), &patterns));
    }
}
