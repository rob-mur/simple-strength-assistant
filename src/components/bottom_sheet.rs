use dioxus::prelude::*;

/// Visual variant for a bottom-sheet item.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum BottomSheetVariant {
    #[default]
    Default,
    Danger,
}

/// A single item in the bottom sheet.
#[derive(Clone, Debug, PartialEq)]
pub struct BottomSheetItem {
    pub label: String,
    pub icon: Option<Element>,
    pub variant: BottomSheetVariant,
    /// Optional custom `data-testid`. Falls back to `bottom-sheet-item-{idx}`.
    pub testid: Option<String>,
}

/// A bottom sheet pinned to the viewport bottom with full-width tap targets.
///
/// - `items`: list of actions to display.
/// - `on_select`: called with the index of the tapped item.
/// - `on_dismiss`: called when the backdrop is tapped.
#[component]
pub fn BottomSheet(
    items: Vec<BottomSheetItem>,
    on_select: EventHandler<usize>,
    on_dismiss: EventHandler<()>,
) -> Element {
    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 bg-black/50 z-50",
            "data-testid": "bottom-sheet-backdrop",
            onclick: move |_| on_dismiss.call(()),

            // Sheet container — stop propagation so tapping items doesn't dismiss
            div {
                class: "fixed bottom-0 left-0 right-0 bg-base-100 rounded-t-2xl z-50 pb-safe",
                "data-testid": "bottom-sheet",
                onclick: move |e| e.stop_propagation(),

                // Drag handle
                div {
                    class: "flex justify-center py-3",
                    div { class: "w-10 h-1 rounded-full bg-base-300" }
                }

                // Items
                div {
                    class: "flex flex-col pb-4",
                    for (idx, item) in items.iter().enumerate() {
                        {
                            let label = item.label.clone();
                            let is_danger = item.variant == BottomSheetVariant::Danger;
                            let testid = item.testid.clone().unwrap_or_else(|| format!("bottom-sheet-item-{idx}"));

                            rsx! {
                                button {
                                    key: "{idx}",
                                    class: {
                                        let base = "flex items-center gap-3 w-full min-h-[56px] px-6 text-lg font-medium active:bg-base-200 transition-colors";
                                        if is_danger {
                                            format!("{base} text-error")
                                        } else {
                                            base.to_string()
                                        }
                                    },
                                    "data-testid": "{testid}",
                                    onclick: move |_| on_select.call(idx),
                                    span { "{label}" }
                                }
                            }
                        }
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
    fn bottom_sheet_item_default_variant() {
        let item = BottomSheetItem {
            label: "Test".to_string(),
            icon: None,
            variant: BottomSheetVariant::Default,
            testid: None,
        };
        assert_eq!(item.variant, BottomSheetVariant::Default);
    }

    #[test]
    fn bottom_sheet_item_danger_variant() {
        let item = BottomSheetItem {
            label: "Delete".to_string(),
            icon: None,
            variant: BottomSheetVariant::Danger,
            testid: None,
        };
        assert_eq!(item.variant, BottomSheetVariant::Danger);
    }

    #[test]
    fn bottom_sheet_items_have_correct_labels() {
        let items = vec![
            BottomSheetItem {
                label: "View History".to_string(),
                icon: None,
                variant: BottomSheetVariant::Default,
                testid: None,
            },
            BottomSheetItem {
                label: "Cancel".to_string(),
                icon: None,
                variant: BottomSheetVariant::Default,
                testid: None,
            },
        ];
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].label, "View History");
        assert_eq!(items[1].label, "Cancel");
    }
}
