// Imports
use crate::RnApp;
use crate::theme::{self, Theme};
use adw::prelude::*;
use gtk4::{gio, glib, glib::clone};
use tracing::error;

impl RnApp {
    pub(crate) fn setup_actions(&self) {
        let action_quit = gio::SimpleAction::new("quit", None);
        self.add_action(&action_quit);

        let action_color_scheme = gio::SimpleAction::new_stateful(
            "color-scheme",
            Some(&glib::VariantType::new("s").unwrap()),
            &String::from("system").to_variant(),
        );
        self.add_action(&action_color_scheme);

        let action_new_window = gio::SimpleAction::new("new-window", None);
        self.add_action(&action_new_window);

        action_color_scheme
            .bind_property("state", &self.style_manager(), "color-scheme")
            .transform_to(move |_, val: glib::Variant| {
                match val.get::<String>().unwrap().as_str() {
                    "default" => Some(adw::ColorScheme::Default),
                    "force-light" => Some(adw::ColorScheme::ForceLight),
                    "force-dark" => Some(adw::ColorScheme::ForceDark),
                    _ => None,
                }
            })
            .transform_from(move |_, val: adw::ColorScheme| match val {
                // the action state is a GVariant, not a plain string
                adw::ColorScheme::Default => Some("default".to_variant().to_value()),
                adw::ColorScheme::ForceLight => Some("force-light".to_variant().to_value()),
                adw::ColorScheme::ForceDark => Some("force-dark".to_variant().to_value()),
                _ => None,
            })
            .sync_create()
            .bidirectional()
            .build();

        let action_theme = gio::SimpleAction::new_stateful(
            "theme",
            Some(&glib::VariantType::new("s").unwrap()),
            &Theme::System.id().to_variant(),
        );
        self.add_action(&action_theme);

        action_theme.connect_change_state(clone!(
            #[weak(rename_to = app)]
            self,
            move |action, state| {
                let Some(id) = state.and_then(|s| s.get::<String>()) else {
                    return;
                };
                let Some(theme) = Theme::from_id(&id) else {
                    error!("Unknown theme `{id}`");
                    return;
                };
                let app_settings = app.app_settings();
                // The color scheme to return to when switching to the system theme
                let system_scheme = match app_settings
                    .as_ref()
                    .map(|s| s.string("color-scheme"))
                    .as_deref()
                {
                    Some("force-light") => adw::ColorScheme::ForceLight,
                    Some("force-dark") => adw::ColorScheme::ForceDark,
                    _ => adw::ColorScheme::Default,
                };
                theme::apply(theme, system_scheme);
                action.set_state(&id.to_variant());

                if let Some(app_settings) = app_settings
                    && let Err(e) = app_settings.set_string("theme", &id)
                {
                    error!("Failed to set setting `theme`, Err: {e:?}");
                }
            }
        ));

        // Quit App
        action_quit.connect_activate(clone!(
            #[weak(rename_to = app)]
            self,
            move |_, _| {
                // Request closing all windows. They then get the chance to display a save dialog on unsaved changes
                for appwindow in app.windows() {
                    appwindow.close();
                }

                if app.windows().is_empty() {
                    app.quit();
                }
            }
        ));

        action_new_window.connect_activate(clone!(
            #[weak(rename_to = app)]
            self,
            move |_, _| {
                app.new_appwindow_init_show();
            }
        ));
    }

    // Accelerators / Keyboard Shortcuts
    pub(crate) fn setup_action_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Ctrl>q"]);
        self.set_accels_for_action("app.new-window", &["<Ctrl>n"]);
    }
}
