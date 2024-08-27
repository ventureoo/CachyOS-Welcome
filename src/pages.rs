use crate::application_browser::ApplicationBrowser;
use crate::data_types::*;
use crate::utils::PacmanWrapper;
use crate::{fl, utils};

use std::boxed::Box;
use std::fmt::Write;
use std::path::Path;
use std::str;
use std::sync::Mutex;

use glib::translate::FromGlib;
use gtk::{glib, Builder};
use once_cell::sync::Lazy;
use phf::phf_ordered_map;

use gtk::prelude::*;

use subprocess::{Exec, Redirection};

#[macro_export]
macro_rules! create_gtk_button {
    ($message_id:literal) => {{
        let temp_btn = gtk::Button::with_label(&fl!($message_id));
        temp_btn.set_widget_name($message_id);
        temp_btn
    }};
}

#[macro_export]
macro_rules! create_tweak_checkbox {
    ($tweak_msg:literal,$action_data:literal,$action_type:literal,$alpm_pkg_name:literal) => {{
        let temp_btn =
            gtk::CheckButton::with_label(&fl!("tweak-enabled-title", tweak = $tweak_msg));
        temp_btn.set_widget_name($tweak_msg);

        set_tweak_check_data(&temp_btn, $action_data, $action_type, $alpm_pkg_name);
        connect_tweak(&temp_btn, $action_data);
        temp_btn
    }};
}

static G_LOCAL_UNITS: Lazy<Mutex<SystemdUnits>> = Lazy::new(|| Mutex::new(SystemdUnits::new()));
static G_GLOBAL_UNITS: Lazy<Mutex<SystemdUnits>> = Lazy::new(|| Mutex::new(SystemdUnits::new()));

static G_DNS_SERVERS: phf::OrderedMap<&'static str, &'static str> = phf_ordered_map! {
    "Adguard" => "94.140.14.14",
    "Adguard Family Protection" => "94.140.14.15",
    "Cloudflare" => "1.1.1.1",
    "Cloudflare Malware and adult content blocking" => "1.1.1.3",
    "DNS.Watch" => "84.200.69.80",
    "Cisco Umbrella(OpenDNS)" => "208.67.222.222,208.67.220.220",
    "Quad9" => "9.9.9.9",
    "Google" => "8.8.8.8,8.8.4.4",
    "Yandex" => "77.88.8.8,77.88.8.1",
};

struct DialogMessage {
    pub msg: String,
    pub msg_type: gtk::MessageType,
    pub action: Action,
}

enum Action {
    RemoveLock,
    RemoveOrphans,
    SetDnsServer,
    InstallGaming,
    InstallSnapper,
}

fn update_translation_apps_section(section_box: &gtk::Box) {
    for section_box_element in section_box.children() {
        if let Ok(section_label) = section_box_element.clone().downcast::<gtk::Label>() {
            section_label.set_text(&fl!("applications"));
        }
    }
}

fn update_translation_fixes_section(section_box: &gtk::Box) {
    for section_box_element in section_box.children() {
        if let Ok(button_box) = section_box_element.clone().downcast::<gtk::Box>() {
            for button_box_widget in button_box.children() {
                let box_element_btn = button_box_widget.downcast::<gtk::Button>().unwrap();
                let widget_name = box_element_btn.widget_name();
                let translated_text = crate::localization::get_locale_text(&widget_name);
                box_element_btn.set_label(&translated_text);
            }
        } else if let Ok(section_label) = section_box_element.downcast::<gtk::Label>() {
            section_label.set_text(&fl!("fixes"));
        }
    }
}

fn update_translation_connections_section(section_box: &gtk::Box) {
    for section_box_element in section_box.children() {
        if let Ok(object_box) = section_box_element.clone().downcast::<gtk::Box>() {
            for object_box_widget in object_box.children() {
                let widget_name = object_box_widget.widget_name();
                if let Ok(box_element_btn) = object_box_widget.clone().downcast::<gtk::Button>() {
                    let translated_text = crate::localization::get_locale_text(&widget_name);
                    box_element_btn.set_label(&translated_text);
                } else if let Ok(box_element_label) = object_box_widget.downcast::<gtk::Label>() {
                    let translated_text = crate::localization::get_locale_text(&widget_name);
                    box_element_label.set_text(&translated_text);
                }
            }
        } else if let Ok(section_label) = section_box_element.downcast::<gtk::Label>() {
            section_label.set_text(&fl!("dns-settings"));
        }
    }
}

fn update_translation_options_section(section_box: &gtk::Box) {
    for section_box_element in section_box.children() {
        if let Ok(button_box) = section_box_element.clone().downcast::<gtk::Box>() {
            for button_box_widget in button_box.children() {
                let box_element_btn = button_box_widget.downcast::<gtk::Button>().unwrap();
                let widget_name = box_element_btn.widget_name().to_string();
                let translated_text = fl!("tweak-enabled-title", tweak = widget_name);
                box_element_btn.set_label(&translated_text);
            }
        } else if let Ok(section_label) = section_box_element.downcast::<gtk::Label>() {
            section_label.set_text(&fl!("tweaks"));
        }
    }
}

pub fn update_translations(builder: &Builder) {
    // Update buttons
    let tweakbrowser_btn: gtk::Button = builder.object("tweaksBrowser").unwrap();
    tweakbrowser_btn.set_label(&fl!("tweaksbrowser-label"));

    let appbrowser_btn: gtk::Button = builder.object("appBrowser").unwrap();
    appbrowser_btn.set_label(&fl!("appbrowser-label"));

    let stack: gtk::Stack = builder.object("stack").unwrap();
    {
        if let Some(widget) = stack.child_by_name("tweaksBrowserpage") {
            if let Ok(viewport) = widget.downcast::<gtk::Viewport>() {
                let second_child =
                    &viewport.children()[0].clone().downcast::<gtk::Box>().unwrap().children()[1]
                        .clone()
                        .downcast::<gtk::Box>()
                        .unwrap();

                for second_child_child_widget in second_child.children() {
                    let second_child_child_box =
                        second_child_child_widget.downcast::<gtk::Box>().unwrap();

                    match second_child_child_box.widget_name().as_str() {
                        "tweaksBrowserpage_options" => {
                            update_translation_options_section(&second_child_child_box)
                        },
                        "tweaksBrowserpage_fixes" => {
                            update_translation_fixes_section(&second_child_child_box)
                        },
                        "tweaksBrowserpage_apps" => {
                            update_translation_apps_section(&second_child_child_box)
                        },
                        _ => panic!("Unknown widget!"),
                    }
                }
            }
        }
        if let Some(widget) = stack.child_by_name("dnsConnectionsBrowserpage") {
            if let Ok(viewport) = widget.downcast::<gtk::Viewport>() {
                let second_child =
                    &viewport.children()[0].clone().downcast::<gtk::Box>().unwrap().children()[1]
                        .clone()
                        .downcast::<gtk::Box>()
                        .unwrap();

                for second_child_child_widget in second_child.children() {
                    let second_child_child_box =
                        second_child_child_widget.downcast::<gtk::Box>().unwrap();
                    update_translation_connections_section(&second_child_child_box);
                }
            }
        }
        if let Some(widget) = stack.child_by_name("appBrowserpage") {
            if let Ok(viewport) = widget.downcast::<gtk::Viewport>() {
                let first_child = &viewport.children()[0].clone().downcast::<gtk::Box>().unwrap();
                for first_child_box in first_child.children() {
                    if first_child_box.widget_name() != "appBrowserpageimpl" {
                        if let Ok(child_scrolledwindow) =
                            &first_child_box.downcast::<gtk::Grid>().unwrap().children()[0]
                                .clone()
                                .downcast::<gtk::ScrolledWindow>()
                        {
                            let tree_view = &child_scrolledwindow.children()[0]
                                .clone()
                                .downcast::<gtk::TreeView>()
                                .unwrap();
                            for tree_column in &tree_view.columns() {
                                if tree_column.title().unwrap().is_empty() {
                                    continue;
                                }
                                let column_name =
                                    unsafe { *tree_column.data::<&str>("name").unwrap().as_ptr() };
                                if column_name.is_empty() {
                                    continue;
                                }

                                let translated_text =
                                    crate::localization::get_locale_text(column_name);
                                tree_column.set_title(&translated_text);
                            }
                        }
                        continue;
                    }
                    let appbrowserimpl = &first_child_box.clone().downcast::<gtk::Box>().unwrap();
                    for box_element in appbrowserimpl.children() {
                        if let Ok(box_element_btn) = box_element.clone().downcast::<gtk::Button>() {
                            let widget_name = box_element_btn.widget_name();
                            let translated_text =
                                crate::localization::get_locale_text(&widget_name);
                            box_element_btn.set_label(&translated_text);
                        }
                    }
                }
            }
        }
    }
}

fn set_tweak_check_data(
    check_btn: &gtk::CheckButton,
    action_data: &'static str,
    action_type: &'static str,
    alpm_package_name: &'static str,
) {
    unsafe {
        check_btn.set_data("actionData", action_data);
        check_btn.set_data("actionType", action_type);
        check_btn.set_data("alpmPackage", alpm_package_name);
    }
}

fn connect_tweak(check_btn: &gtk::CheckButton, action_data: &'static str) {
    let action_data_str = action_data.to_owned();
    if G_LOCAL_UNITS.lock().unwrap().enabled_units.contains(&action_data_str)
        || G_GLOBAL_UNITS.lock().unwrap().enabled_units.contains(&action_data_str)
    {
        check_btn.set_active(true);
    }
    connect_clicked_and_save(check_btn, on_servbtn_clicked);
}

fn get_nm_connections() -> Vec<String> {
    let connections = Exec::cmd("/sbin/nmcli")
        .args(&["-t", "-f", "NAME", "connection", "show"])
        .stdout(Redirection::Pipe)
        .capture()
        .unwrap()
        .stdout_str();

    // get list of connections separated by newline
    connections.split('\n').filter(|x| !x.is_empty()).map(String::from).collect::<Vec<_>>()
}

fn launch_kwin_debug_window() {
    let _ = Exec::cmd("qdbus6")
        .args(&["org.kde.KWin", "/KWin", "org.kde.KWin.showDebugConsole"])
        .join()
        .unwrap();
}

fn create_fixes_section(builder: &Builder) -> gtk::Box {
    let topbox = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let button_box_f = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let button_box_s = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let button_box_t = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let button_box_frth = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let label = gtk::Label::new(None);
    label.set_line_wrap(true);
    label.set_justify(gtk::Justification::Center);
    label.set_text(&fl!("fixes"));

    let removelock_btn = create_gtk_button!("remove-lock-title");
    let reinstall_btn = create_gtk_button!("reinstall-title");
    let refreshkeyring_btn = create_gtk_button!("refresh-keyrings-title");
    let update_system_btn = create_gtk_button!("update-system-title");
    let remove_orphans_btn = create_gtk_button!("remove-orphans-title");
    let clear_pkgcache_btn = create_gtk_button!("clear-pkgcache-title");
    let rankmirrors_btn = create_gtk_button!("rankmirrors-title");

    let install_gaming_btn = create_gtk_button!("install-gaming-title");
    let install_snapper_btn = create_gtk_button!("install-snapper-title");
    let install_spoof_dpi_btn = create_gtk_button!("install-spoof-dpi-title");

    // Create context channel.
    let (dialog_tx, dialog_rx) = glib::MainContext::channel(glib::Priority::default());

    // Connect signals.
    let dialog_tx_clone = dialog_tx.clone();
    let dialog_tx_gaming = dialog_tx.clone();
    let dialog_tx_snapper = dialog_tx.clone();
    let dialog_tx_spoof = dialog_tx.clone();
    removelock_btn.connect_clicked(move |_| {
        let dialog_tx_clone = dialog_tx_clone.clone();
        std::thread::spawn(move || {
            if Path::new("/var/lib/pacman/db.lck").exists() {
                let _ = Exec::cmd("/sbin/pkexec")
                    .arg("bash")
                    .arg("-c")
                    .arg("rm /var/lib/pacman/db.lck")
                    .join()
                    .unwrap();
                if !Path::new("/var/lib/pacman/db.lck").exists() {
                    dialog_tx_clone
                        .send(DialogMessage {
                            msg: fl!("removed-db-lock"),
                            msg_type: gtk::MessageType::Info,
                            action: Action::RemoveLock,
                        })
                        .expect("Couldn't send data to channel");
                }
            } else {
                dialog_tx_clone
                    .send(DialogMessage {
                        msg: fl!("lock-doesnt-exist"),
                        msg_type: gtk::MessageType::Info,
                        action: Action::RemoveLock,
                    })
                    .expect("Couldn't send data to channel");
            }
        });
    });
    reinstall_btn.connect_clicked(move |_| {
        // Spawn child process in separate thread.
        std::thread::spawn(move || {
            let _ = utils::run_cmd_terminal(String::from("pacman -S $(pacman -Qnq)"), true);
        });
    });
    refreshkeyring_btn.connect_clicked(on_refreshkeyring_btn_clicked);
    update_system_btn.connect_clicked(on_update_system_btn_clicked);
    remove_orphans_btn.connect_clicked(move |_| {
        // Spawn child process in separate thread.
        let dialog_tx_clone = dialog_tx.clone();
        std::thread::spawn(move || {
            // check if you have orphans packages.
            let mut orphan_pkgs = Exec::cmd("/sbin/pacman")
                .arg("-Qtdq")
                .stdout(Redirection::Pipe)
                .capture()
                .unwrap()
                .stdout_str();

            // get list of packages separated by space,
            // and check if it's empty or not.
            orphan_pkgs = orphan_pkgs.replace('\n', " ");
            if orphan_pkgs.is_empty() {
                dialog_tx_clone
                    .send(DialogMessage {
                        msg: fl!("orphans-not-found"),
                        msg_type: gtk::MessageType::Info,
                        action: Action::RemoveOrphans,
                    })
                    .expect("Couldn't send data to channel");
                return;
            }
            let _ = utils::run_cmd_terminal(format!("pacman -Rns {orphan_pkgs}"), true);
        });
    });
    clear_pkgcache_btn.connect_clicked(on_clear_pkgcache_btn_clicked);
    rankmirrors_btn.connect_clicked(move |_| {
        // Spawn child process in separate thread.
        std::thread::spawn(move || {
            let _ = utils::run_cmd_terminal(String::from("cachyos-rate-mirrors"), true);
        });
    });
    install_gaming_btn.connect_clicked(move |_| {
        let dialog_tx_gaming = dialog_tx_gaming.clone();
        // Spawn child process in separate thread.
        std::thread::spawn(move || {
            const alpm_package_name: &str = "cachyos-gaming-meta";
            if !utils::is_alpm_pkg_installed(alpm_package_name) {
                let _ = utils::run_cmd_terminal(format!("pacman -S {alpm_package_name}"), true);
            } else {
                dialog_tx_gaming
                    .send(DialogMessage {
                        msg: fl!("gaming-package-installed"),
                        msg_type: gtk::MessageType::Info,
                        action: Action::InstallGaming,
                    })
                    .expect("Couldn't send data to channel");
            }
        });
    });
    install_snapper_btn.connect_clicked(move |_| {
        let dialog_tx_gaming = dialog_tx_snapper.clone();
        // Spawn child process in separate thread.
        std::thread::spawn(move || {
            const alpm_package_name: &str = "cachyos-snapper-support";
            if !utils::is_alpm_pkg_installed(alpm_package_name) {
                let _ = utils::run_cmd_terminal(format!("pacman -S {alpm_package_name}"), true);
            } else {
                dialog_tx_gaming
                    .send(DialogMessage {
                        msg: fl!("snapper-package-installed"),
                        msg_type: gtk::MessageType::Info,
                        action: Action::InstallSnapper,
                    })
                    .expect("Couldn't send data to channel");
            }
        });
    });
    install_spoof_dpi_btn.connect_clicked(move |_| {
        let dialog_tx_spoof_dpi = dialog_tx_spoof.clone();
        // Spawn child process in separate thread.
        std::thread::spawn(move || {
            const alpm_package_name: &str = "spoof-dpi-bin";
            if !utils::is_alpm_pkg_installed(alpm_package_name) {
                let _ = utils::run_cmd_terminal(format!("pacman -S {alpm_package_name}"), true);
            } else {
                dialog_tx_spoof_dpi
                    .send(DialogMessage {
                        msg: fl!("spoof-dpi-package-installed"),
                        msg_type: gtk::MessageType::Info,
                        action: Action::InstallSnapper,
                    })
                    .expect("Couldn't send data to channel");
            }
        });
    });

    // Setup receiver.
    let removelock_btn_clone = removelock_btn.clone();
    let remove_orphans_btn_clone = remove_orphans_btn.clone();
    let install_gaming_btn_clone = install_gaming_btn.clone();
    let install_snapper_btn_clone = install_snapper_btn.clone();
    dialog_rx.attach(None, move |msg| {
        let widget_obj = match msg.action {
            Action::RemoveLock => &removelock_btn_clone,
            Action::RemoveOrphans => &remove_orphans_btn_clone,
            Action::InstallGaming => &install_gaming_btn_clone,
            Action::InstallSnapper => &install_snapper_btn_clone,
            _ => panic!("Unexpected action!!"),
        };
        let widget_window =
            utils::get_window_from_widget(widget_obj).expect("Failed to retrieve window");

        utils::show_simple_dialog(&widget_window, msg.msg_type, &msg.msg, msg.msg_type.to_string());
        glib::ControlFlow::Continue
    });

    topbox.pack_start(&label, true, false, 1);
    button_box_f.pack_start(&update_system_btn, true, true, 2);
    button_box_f.pack_start(&reinstall_btn, true, true, 2);
    button_box_f.pack_end(&refreshkeyring_btn, true, true, 2);
    button_box_s.pack_start(&removelock_btn, true, true, 2);
    button_box_s.pack_start(&clear_pkgcache_btn, true, true, 2);
    button_box_s.pack_end(&remove_orphans_btn, true, true, 2);
    button_box_t.pack_end(&rankmirrors_btn, true, true, 2);
    if utils::is_root_on_btrfs() {
        button_box_t.pack_end(&install_snapper_btn, true, true, 2);
    }
    button_box_t.pack_end(&install_gaming_btn, true, true, 2);
    button_box_frth.pack_end(&install_spoof_dpi_btn, true, true, 2);

    if Path::new("/usr/bin/nmcli").exists() {
        let dnsserver_btn = create_gtk_button!("dnsserver-title");
        dnsserver_btn.connect_clicked(glib::clone!(@weak builder => move |_| {
            let name = "dnsConnectionsBrowser";
            let stack: gtk::Stack = builder.object("stack").unwrap();
            stack.set_visible_child_name(&format!("{name}page"));
        }));
        button_box_frth.pack_end(&dnsserver_btn, true, true, 2);
    }

    button_box_f.set_halign(gtk::Align::Fill);
    button_box_s.set_halign(gtk::Align::Fill);
    button_box_t.set_halign(gtk::Align::Fill);
    button_box_frth.set_halign(gtk::Align::Fill);
    topbox.pack_end(&button_box_frth, true, true, 5);
    topbox.pack_end(&button_box_t, true, true, 5);
    topbox.pack_end(&button_box_s, true, true, 5);
    topbox.pack_end(&button_box_f, true, true, 5);

    if let Ok(pgrep_res) =
        Exec::cmd("pgrep").args(&["kwin_wayland"]).stdout(subprocess::NullFile).join()
    {
        if pgrep_res.success() {
            let kwinw_debug_btn = create_gtk_button!("show-kwinw-debug-title");
            kwinw_debug_btn.connect_clicked(move |_| {
                // Spawn child process in separate thread.
                std::thread::spawn(move || {
                    // do we even need to start that in separate thread. should be fine without
                    launch_kwin_debug_window();
                });
            });
            button_box_frth.pack_end(&kwinw_debug_btn, true, true, 2);
        }
    }

    topbox.set_hexpand(true);
    topbox
}

fn create_options_section() -> gtk::Box {
    let topbox = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let box_collection = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let box_collection_s = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let label = gtk::Label::new(None);
    label.set_line_wrap(true);
    label.set_justify(gtk::Justification::Center);
    label.set_text(&fl!("tweaks"));

    let psd_btn = create_tweak_checkbox!(
        "Profile-sync-daemon",
        "psd.service",
        "user_service",
        "profile-sync-daemon"
    );
    let systemd_oomd_btn =
        create_tweak_checkbox!("Systemd-oomd", "systemd-oomd.service", "service", "");
    let bpftune_btn =
        create_tweak_checkbox!("Bpftune", "bpftune.service", "service", "bpftune-git");
    let bluetooth_btn =
        create_tweak_checkbox!("Bluetooth", "bluetooth.service", "service", "bluez");
    let ananicy_cpp_btn =
        create_tweak_checkbox!("Ananicy Cpp", "ananicy-cpp.service", "service", "ananicy-cpp");

    topbox.pack_start(&label, true, false, 1);
    box_collection.pack_start(&psd_btn, true, false, 2);
    box_collection_s.pack_start(&systemd_oomd_btn, true, false, 2);
    box_collection_s.pack_start(&bpftune_btn, true, false, 2);
    box_collection.pack_start(&ananicy_cpp_btn, true, false, 2);
    box_collection_s.pack_start(&bluetooth_btn, true, false, 2);
    box_collection.set_halign(gtk::Align::Fill);
    box_collection_s.set_halign(gtk::Align::Fill);
    topbox.pack_end(&box_collection_s, true, false, 1);
    topbox.pack_end(&box_collection, true, false, 1);

    topbox.set_hexpand(true);
    topbox
}

fn create_apps_section() -> Option<gtk::Box> {
    let topbox = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let box_collection = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let label = gtk::Label::new(None);
    label.set_line_wrap(true);
    label.set_justify(gtk::Justification::Center);
    label.set_text(&fl!("applications"));

    // Check first btn.
    if Path::new("/sbin/cachyos-pi").exists() {
        let cachyos_pi = gtk::Button::with_label("CachyOS PackageInstaller");
        cachyos_pi.connect_clicked(on_appbtn_clicked);
        box_collection.pack_start(&cachyos_pi, true, true, 2);
    }
    // Check second btn.
    if Path::new("/sbin/cachyos-kernel-manager").exists() {
        let cachyos_km = gtk::Button::with_label("CachyOS Kernel Manager");
        cachyos_km.connect_clicked(on_appbtn_clicked);
        box_collection.pack_start(&cachyos_km, true, true, 2);
    }

    topbox.pack_start(&label, true, true, 5);

    box_collection.set_halign(gtk::Align::Fill);
    topbox.pack_end(&box_collection, true, true, 0);

    topbox.set_hexpand(true);
    match !box_collection.children().is_empty() {
        true => Some(topbox),
        _ => None,
    }
}

fn create_connections_section() -> gtk::Box {
    let topbox = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let connection_box = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    let dnsservers_box = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    let label = gtk::Label::new(None);
    label.set_line_wrap(true);
    label.set_justify(gtk::Justification::Center);
    label.set_text(&fl!("dns-settings"));

    let connections_label = gtk::Label::new(None);
    connections_label.set_justify(gtk::Justification::Left);
    connections_label.set_text(&fl!("select-connection"));
    connections_label.set_widget_name("select-connection");
    let servers_label = gtk::Label::new(None);
    servers_label.set_justify(gtk::Justification::Left);
    servers_label.set_text(&fl!("select-dns-server"));
    servers_label.set_widget_name("select-dns-server");
    let apply_btn = create_gtk_button!("apply");
    let reset_btn = create_gtk_button!("reset");

    let combo_conn = {
        let store = gtk::ListStore::new(&[String::static_type()]);
        let nm_connections = get_nm_connections();
        for nm_connection in nm_connections.iter() {
            store.set(&store.append(), &[(0, nm_connection)]);
        }
        utils::create_combo_with_model(&store)
    };
    let combo_servers = {
        let store = gtk::ListStore::new(&[String::static_type()]);
        for dns_server in G_DNS_SERVERS.keys() {
            store.set(&store.append(), &[(0, dns_server)]);
        }
        utils::create_combo_with_model(&store)
    };
    combo_servers.set_active(Some(2));

    combo_conn.set_widget_name("connections_combo");
    combo_servers.set_widget_name("servers_combo");

    // Create context channel.
    let (dialog_tx, dialog_rx) = glib::MainContext::channel(glib::Priority::default());

    // Connect signals.
    let dialog_tx_clone = dialog_tx.clone();
    let combo_conn_clone = combo_conn.clone();
    let combo_serv_clone = combo_servers.clone();
    apply_btn.connect_clicked(move |_| {
        let dialog_tx_clone = dialog_tx_clone.clone();
        let conn_name = {
            if let Some(tree_iter) = combo_conn_clone.active_iter() {
                let model = combo_conn_clone.model().unwrap();
                let group_gobj = model.value(&tree_iter, 0);
                let group = group_gobj.get::<&str>().unwrap();
                String::from(group)
            } else {
                "".into()
            }
        };
        let server_name = {
            if let Some(tree_iter) = combo_serv_clone.active_iter() {
                let model = combo_serv_clone.model().unwrap();
                let group_gobj = model.value(&tree_iter, 0);
                let group = group_gobj.get::<&str>().unwrap();
                String::from(group)
            } else {
                "".into()
            }
        };
        let server_addr = G_DNS_SERVERS.get(&server_name).unwrap();
        std::thread::spawn(move || {
            let status_code = Exec::cmd("/sbin/pkexec")
                .arg("bash")
                .arg("-c")
                .arg(format!(
                    "nmcli con mod '{conn_name}' ipv4.dns '{server_addr}' && systemctl restart \
                     NetworkManager"
                ))
                .join()
                .unwrap();
            if status_code.success() {
                dialog_tx_clone
                    .send(DialogMessage {
                        msg: fl!("dns-server-changed"),
                        msg_type: gtk::MessageType::Info,
                        action: Action::SetDnsServer,
                    })
                    .expect("Couldn't send data to channel");
            } else {
                dialog_tx_clone
                    .send(DialogMessage {
                        msg: fl!("dns-server-failed"),
                        msg_type: gtk::MessageType::Error,
                        action: Action::SetDnsServer,
                    })
                    .expect("Couldn't send data to channel");
            }
        });
    });
    let dialog_tx_clone = dialog_tx.clone();
    let combo_conn_clone = combo_conn.clone();
    reset_btn.connect_clicked(move |_| {
        let dialog_tx_clone = dialog_tx_clone.clone();
        let conn_name = {
            if let Some(tree_iter) = combo_conn_clone.active_iter() {
                let model = combo_conn_clone.model().unwrap();
                let group_gobj = model.value(&tree_iter, 0);
                let group = group_gobj.get::<&str>().unwrap();
                String::from(group)
            } else {
                "".into()
            }
        };
        std::thread::spawn(move || {
            let status_code = Exec::cmd("/sbin/pkexec")
                .arg("bash")
                .arg("-c")
                .arg(format!(
                    "nmcli con mod '{conn_name}' ipv4.dns '' && systemctl restart NetworkManager"
                ))
                .join()
                .unwrap();
            if status_code.success() {
                dialog_tx_clone
                    .send(DialogMessage {
                        msg: fl!("dns-server-reset"),
                        msg_type: gtk::MessageType::Info,
                        action: Action::SetDnsServer,
                    })
                    .expect("Couldn't send data to channel");
            } else {
                dialog_tx_clone
                    .send(DialogMessage {
                        msg: fl!("dns-server-reset-failed"),
                        msg_type: gtk::MessageType::Error,
                        action: Action::SetDnsServer,
                    })
                    .expect("Couldn't send data to channel");
            }
        });
    });

    // Setup receiver
    let apply_btn_clone = apply_btn.clone();
    dialog_rx.attach(None, move |msg| {
        let widget_obj = &apply_btn_clone;
        let widget_window =
            utils::get_window_from_widget(widget_obj).expect("Failed to retrieve window");

        utils::show_simple_dialog(&widget_window, msg.msg_type, &msg.msg, msg.msg_type.to_string());
        glib::ControlFlow::Continue
    });

    topbox.pack_start(&label, true, false, 1);
    connection_box.pack_start(&connections_label, true, true, 2);
    connection_box.pack_end(&combo_conn, true, true, 2);
    dnsservers_box.pack_start(&servers_label, true, true, 2);
    dnsservers_box.pack_end(&combo_servers, true, true, 2);
    button_box.pack_start(&reset_btn, true, true, 2);
    button_box.pack_end(&apply_btn, true, true, 2);
    connection_box.set_halign(gtk::Align::Fill);
    dnsservers_box.set_halign(gtk::Align::Fill);
    button_box.set_halign(gtk::Align::Fill);
    topbox.pack_start(&connection_box, true, true, 5);
    topbox.pack_start(&dnsservers_box, true, true, 5);
    topbox.pack_start(&button_box, true, true, 5);

    topbox.set_hexpand(true);
    topbox
}

fn load_enabled_units() {
    G_LOCAL_UNITS.lock().unwrap().loaded_units.clear();
    G_LOCAL_UNITS.lock().unwrap().enabled_units.clear();

    let mut exec_out = Exec::shell("systemctl list-unit-files -q --no-pager | tr -s \" \"")
        .stdout(Redirection::Pipe)
        .capture()
        .unwrap()
        .stdout_str();
    exec_out.pop();

    let service_list = exec_out.split('\n');

    for service in service_list {
        let out: Vec<&str> = service.split(' ').collect();
        G_LOCAL_UNITS.lock().unwrap().loaded_units.push(String::from(out[0]));
        if out[1] == "enabled" {
            G_LOCAL_UNITS.lock().unwrap().enabled_units.push(String::from(out[0]));
        }
    }
}

fn load_global_enabled_units() {
    G_GLOBAL_UNITS.lock().unwrap().loaded_units.clear();
    G_GLOBAL_UNITS.lock().unwrap().enabled_units.clear();

    let mut exec_out = Exec::shell("systemctl --user list-unit-files -q --no-pager | tr -s \" \"")
        .stdout(Redirection::Pipe)
        .capture()
        .unwrap()
        .stdout_str();
    exec_out.pop();

    let service_list = exec_out.split('\n');
    for service in service_list {
        let out: Vec<&str> = service.split(' ').collect();
        G_GLOBAL_UNITS.lock().unwrap().loaded_units.push(String::from(out[0]));
        if out[1] == "enabled" {
            G_GLOBAL_UNITS.lock().unwrap().enabled_units.push(String::from(out[0]));
        }
    }
}

pub fn create_tweaks_page(builder: &Builder) {
    let install: gtk::Button = builder.object("tweaksBrowser").unwrap();
    install.set_visible(true);
    install.set_label(&fl!("tweaksbrowser-label"));

    load_enabled_units();
    load_global_enabled_units();

    let viewport = gtk::Viewport::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    let image = gtk::Image::from_icon_name(Some("go-previous"), gtk::IconSize::Button);
    let back_btn = gtk::Button::new();
    back_btn.set_image(Some(&image));
    back_btn.set_widget_name("home");

    back_btn.connect_clicked(glib::clone!(@weak builder => move |button| {
        let name = button.widget_name();
        let stack: gtk::Stack = builder.object("stack").unwrap();
        stack.set_visible_child_name(&format!("{name}page"));
    }));

    let options_section_box = create_options_section();
    let fixes_section_box = create_fixes_section(builder);
    let apps_section_box_opt = create_apps_section();

    let child_name = "tweaksBrowserpage";
    options_section_box.set_widget_name(&format!("{child_name}_options"));
    fixes_section_box.set_widget_name(&format!("{child_name}_fixes"));
    if apps_section_box_opt.is_some() {
        apps_section_box_opt.as_ref().unwrap().set_widget_name(&format!("{child_name}_apps"));
    }

    let grid = gtk::Grid::new();
    grid.set_hexpand(true);
    grid.set_margin_start(10);
    grid.set_margin_end(10);
    grid.set_margin_top(5);
    grid.set_margin_bottom(5);
    grid.attach(&back_btn, 0, 1, 1, 1);
    let box_collection_s = gtk::Box::new(gtk::Orientation::Vertical, 5);
    let box_collection = gtk::Box::new(gtk::Orientation::Vertical, 5);
    box_collection.set_widget_name(child_name);

    box_collection.pack_start(&options_section_box, false, false, 10);
    box_collection.pack_start(&fixes_section_box, false, false, 10);

    if let Some(apps_section_box) = apps_section_box_opt {
        box_collection.pack_end(&apps_section_box, false, false, 10);
    }

    box_collection.set_valign(gtk::Align::Center);
    box_collection.set_halign(gtk::Align::Center);
    box_collection_s.pack_start(&grid, false, false, 0);
    box_collection_s.pack_start(&box_collection, false, false, 10);
    viewport.add(&box_collection_s);
    viewport.show_all();

    let stack: gtk::Stack = builder.object("stack").unwrap();
    stack.add_named(&viewport, child_name);
}

pub fn create_dnsconnections_page(builder: &Builder) {
    let viewport = gtk::Viewport::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    let image = gtk::Image::from_icon_name(Some("go-previous"), gtk::IconSize::Button);
    let back_btn = gtk::Button::new();
    back_btn.set_image(Some(&image));
    back_btn.set_widget_name("tweaksBrowser");

    back_btn.connect_clicked(glib::clone!(@weak builder => move |button| {
        let name = button.widget_name();
        let stack: gtk::Stack = builder.object("stack").unwrap();
        stack.set_visible_child_name(&format!("{name}page"));
    }));

    let connections_section_box = create_connections_section();

    let child_name = "dnsConnectionsBrowserpage";
    connections_section_box.set_widget_name(&format!("{child_name}_connections"));

    let grid = gtk::Grid::new();
    grid.set_hexpand(true);
    grid.set_margin_start(10);
    grid.set_margin_end(10);
    grid.set_margin_top(5);
    grid.set_margin_bottom(5);
    grid.attach(&back_btn, 0, 1, 1, 1);
    let box_collection_s = gtk::Box::new(gtk::Orientation::Vertical, 5);
    let box_collection = gtk::Box::new(gtk::Orientation::Vertical, 5);
    box_collection.set_widget_name(child_name);

    box_collection.pack_start(&connections_section_box, false, false, 10);

    box_collection.set_valign(gtk::Align::Center);
    box_collection.set_halign(gtk::Align::Center);
    box_collection_s.pack_start(&grid, false, false, 0);
    box_collection_s.pack_start(&box_collection, false, false, 10);
    viewport.add(&box_collection_s);
    viewport.show_all();

    let stack: gtk::Stack = builder.object("stack").unwrap();
    stack.add_named(&viewport, child_name);
}

pub fn create_appbrowser_page(builder: &Builder) {
    let install: gtk::Button = builder.object("appBrowser").unwrap();
    install.set_visible(true);
    install.set_label(&fl!("appbrowser-label"));

    let viewport = gtk::Viewport::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    let back_btn = ApplicationBrowser::back_btn_impl()
        .expect("Failed to get back btn from application browser");
    back_btn.connect_clicked(glib::clone!(@weak builder => move |button| {
        let name = button.widget_name();
        let stack: gtk::Stack = builder.object("stack").unwrap();
        stack.set_visible_child_name(&format!("{name}page"));
    }));
    let app_browser_box =
        ApplicationBrowser::page_impl().expect("Failed to get page of application browser");

    // Add grid to the viewport
    // NOTE: we might eliminate that?
    viewport.add(&app_browser_box);
    viewport.show_all();

    let stack: gtk::Stack = builder.object("stack").unwrap();
    let child_name = "appBrowserpage";
    stack.add_named(&viewport, child_name);
}

fn toggle_service(
    action_type: &str,
    action_data: &str,
    alpm_package_name: &str,
    widget_window: gtk::Window,
    callback: std::boxed::Box<dyn Fn(bool)>,
) {
    let units_handle = if action_type == "user_service" { &G_GLOBAL_UNITS } else { &G_LOCAL_UNITS }
        .lock()
        .unwrap();
    let cmd = if !units_handle.enabled_units.contains(&String::from(action_data)) {
        if action_type == "user_service" {
            format!("systemctl --user enable --now --force {action_data}")
        } else {
            format!("/sbin/pkexec bash -c \"systemctl enable --now --force {action_data}\"")
        }
    } else if action_type == "user_service" {
        format!("systemctl --user disable --now {action_data}")
    } else {
        format!("/sbin/pkexec bash -c \"systemctl disable --now {action_data}\"")
    };

    // Create context channel.
    let (tx, rx) = glib::MainContext::channel(glib::Priority::default());

    let dialog_text = fl!("package-not-installed", package_name = alpm_package_name);

    let action_type = action_type.to_owned();
    let alpm_package_name = alpm_package_name.to_owned();
    // Spawn child process in separate thread.
    std::thread::spawn(move || {
        if !alpm_package_name.is_empty() {
            if !utils::is_alpm_pkg_installed(&alpm_package_name) {
                let _ = utils::run_cmd_terminal(format!("pacman -S {alpm_package_name}"), true);
            }
            if !utils::is_alpm_pkg_installed(&alpm_package_name) {
                tx.send(false).expect("Couldn't send data to channel");
                return;
            }
        }
        Exec::shell(cmd).join().unwrap();

        if action_type == "user_service" {
            load_global_enabled_units();
        } else {
            load_enabled_units();
        }
    });

    rx.attach(None, move |msg| {
        if !msg {
            callback(msg);

            utils::show_simple_dialog(
                &widget_window,
                gtk::MessageType::Error,
                &dialog_text,
                "Error".to_string(),
            );
        }
        glib::ControlFlow::Continue
    });
}

fn on_servbtn_clicked(button: &gtk::CheckButton) {
    // Get action data/type.
    let action_type: &str;
    let action_data: &str;
    let alpm_package_name: &str;
    let signal_handler: u64;
    unsafe {
        action_type = *button.data("actionType").unwrap().as_ptr();
        action_data = *button.data("actionData").unwrap().as_ptr();
        alpm_package_name = *button.data("alpmPackage").unwrap().as_ptr();
        signal_handler = *button.data("signalHandle").unwrap().as_ptr();
    }

    let widget_window = utils::get_window_from_widget(button).expect("Failed to retrieve window");

    let button_sh = button.clone();
    toggle_service(
        action_type,
        action_data,
        alpm_package_name,
        widget_window,
        Box::new(move |msg| {
            let sighandle_id_obj =
                unsafe { glib::signal::SignalHandlerId::from_glib(signal_handler) };
            button_sh.block_signal(&sighandle_id_obj);
            button_sh.set_active(msg);
            button_sh.unblock_signal(&sighandle_id_obj);
        }),
    );
}

fn on_refreshkeyring_btn_clicked(_: &gtk::Button) {
    let pacman = pacmanconf::Config::with_opts(None, Some("/etc/pacman.conf"), Some("/")).unwrap();
    let alpm = alpm_utils::alpm_with_conf(&pacman).unwrap();

    // search local database for packages matching the regex ".*-keyring"
    // e.g pacman -Qq | grep keyring
    let needles: &[String] = &[".*-keyring".into()];
    let found_keyrings = alpm
        .localdb()
        .search(needles.iter())
        .unwrap()
        .into_iter()
        .filter(|pkg| pkg.name() != "gnome-keyring" && pkg.name() != "python-keyring")
        .fold(String::new(), |mut output, pkg| {
            let mut pkgname = String::from(pkg.name());
            pkgname.remove_matches("-keyring");
            let _ = write!(output, "{pkgname} ");
            output
        });

    // Spawn child process in separate thread.
    std::thread::spawn(move || {
        let _ = utils::run_cmd_terminal(
            format!("pacman-key --init && pacman-key --populate {found_keyrings}"),
            true,
        );
    });
}

fn on_update_system_btn_clicked(_: &gtk::Button) {
    let (cmd, escalate) = match utils::get_pacman_wrapper() {
        PacmanWrapper::Aura => ("aura -Syu && aura -Akaxu", false),
        _ => ("pacman -Syu", true),
    };
    // Spawn child process in separate thread.
    std::thread::spawn(move || {
        let _ = utils::run_cmd_terminal(String::from(cmd), escalate);
    });
}

fn on_clear_pkgcache_btn_clicked(_: &gtk::Button) {
    let (cmd, escalate) = match utils::get_pacman_wrapper() {
        PacmanWrapper::Pak => ("pak -Sc", false),
        PacmanWrapper::Yay => ("yay -Sc", false),
        PacmanWrapper::Paru => ("paru -Sc", false),
        _ => ("pacman -Sc", true),
    };
    // Spawn child process in separate thread.
    std::thread::spawn(move || {
        let _ = utils::run_cmd_terminal(String::from(cmd), escalate);
    });
}

fn on_appbtn_clicked(button: &gtk::Button) {
    // Get button label.
    let name = button.label().unwrap();
    let binname = if name == "CachyOS PackageInstaller" {
        "cachyos-pi"
    } else if name == "CachyOS Kernel Manager" {
        "cachyos-kernel-manager"
    } else {
        ""
    };

    // Check if executable exists.
    let exit_status = Exec::cmd("which").arg(binname).join().unwrap();
    if !exit_status.success() {
        return;
    }

    // Get executable path.
    let mut exe_path =
        Exec::cmd("which").arg(binname).stdout(Redirection::Pipe).capture().unwrap().stdout_str();
    exe_path.pop();
    let bash_cmd = format!("{} &disown", &exe_path);

    // Create context channel.
    let (tx, rx) = glib::MainContext::channel(glib::Priority::default());

    // Spawn child process in separate thread.
    std::thread::spawn(move || {
        let exit_status = Exec::shell(bash_cmd).join().unwrap();
        tx.send(format!("Exit status successfully? = {:?}", exit_status.success()))
            .expect("Couldn't send data to channel");
    });

    rx.attach(None, move |text| {
        println!("{text}");
        glib::ControlFlow::Continue
    });
}

fn connect_clicked_and_save<F>(passed_btn: &gtk::CheckButton, callback: F)
where
    F: Fn(&gtk::CheckButton) + 'static,
{
    let sighandle_id = passed_btn.connect_clicked(callback);
    unsafe {
        passed_btn.set_data("signalHandle", sighandle_id.as_raw());
    }
}
