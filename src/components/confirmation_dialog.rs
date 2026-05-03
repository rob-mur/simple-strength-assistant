use dioxus::prelude::*;

/// Visual variant for the confirm button.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ConfirmVariant {
    #[default]
    Default,
    Danger,
}

/// A modal confirmation dialog with Cancel (leftmost) and Confirm buttons.
///
/// - `title`: dialog heading.
/// - `body`: explanatory text.
/// - `confirm_label`: text for the confirm button.
/// - `cancel_label`: text for the cancel button.
/// - `variant`: styling variant for the confirm button.
/// - `on_confirm`: called when the confirm button is tapped.
/// - `on_cancel`: called when the cancel button is tapped.
#[component]
pub fn ConfirmationDialog(
    title: String,
    body: String,
    confirm_label: String,
    cancel_label: String,
    #[props(default)] variant: ConfirmVariant,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4",
            "data-testid": "confirmation-dialog-backdrop",
            onclick: move |_| on_cancel.call(()),

            // Dialog card
            div {
                class: "bg-base-100 rounded-2xl shadow-xl max-w-sm w-full p-6",
                "data-testid": "confirmation-dialog",
                onclick: move |e| e.stop_propagation(),

                h3 {
                    class: "text-lg font-bold mb-2",
                    "data-testid": "confirmation-dialog-title",
                    "{title}"
                }

                p {
                    class: "text-base-content/70 mb-6",
                    "data-testid": "confirmation-dialog-body",
                    "{body}"
                }

                // Button row: Cancel (leftmost), then Confirm
                div {
                    class: "flex gap-3 justify-end",
                    button {
                        class: "btn btn-ghost",
                        "data-testid": "confirmation-dialog-cancel",
                        onclick: move |_| on_cancel.call(()),
                        "{cancel_label}"
                    }
                    button {
                        class: {
                            if variant == ConfirmVariant::Danger {
                                "btn btn-error"
                            } else {
                                "btn btn-primary"
                            }
                        },
                        "data-testid": "confirmation-dialog-confirm",
                        onclick: move |_| on_confirm.call(()),
                        "{confirm_label}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirm_variant_default() {
        let variant = ConfirmVariant::Default;
        assert_eq!(variant, ConfirmVariant::Default);
    }

    #[test]
    fn confirm_variant_danger() {
        let variant = ConfirmVariant::Danger;
        assert_eq!(variant, ConfirmVariant::Danger);
        assert_ne!(variant, ConfirmVariant::Default);
    }
}
