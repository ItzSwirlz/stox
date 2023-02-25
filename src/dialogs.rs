use gettextrs::*;
use gtk4::{prelude::DialogExtManual, traits::GtkWindowExt, *};

fn show_error_dialog(window: &ApplicationWindow, message: &str) {
    let dialog = MessageDialog::builder()
        .transient_for(window)
        .modal(true)
        .buttons(ButtonsType::Ok)
        .text(gettext("Error"))
        .secondary_text(message)
        .message_type(MessageType::Error)
        .build();

    dialog.run_async(|obj, _| obj.close());
}

pub fn show_load_saved_stocks_failed_dialog(window: &ApplicationWindow) {
    show_error_dialog(
        window,
        &gettext("The saved stocks could not be loaded. Try restarting the app.\n\nTo prevent data loss, saving and unsaving stocks will be disabled until this is fixed.",
        ),
    )
}

pub fn show_saving_unsaving_disabled_dialog(window: &ApplicationWindow) {
    show_error_dialog(
        window,
        &gettext(
            "Saving and unsaving stocks is disabled to prevent data loss. Try restarting the app.",
        ),
    );
}

pub fn show_save_stock_failed_dialog(window: &ApplicationWindow) {
    show_error_dialog(
        window,
        &gettext("An error occurred and the stock could not be saved. Try saving it again."),
    )
}

pub fn show_unsave_stock_failed_dialog(window: &ApplicationWindow) {
    show_error_dialog(
        window,
        &gettext("An error occurred and the stock could not be unsaved. Try unsaving it again."),
    );
}
