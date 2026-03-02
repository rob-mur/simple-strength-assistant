use dioxus::prelude::*;

#[component]
pub fn LibraryView() -> Element {
    rsx! {
        div {
            class: "max-w-2xl mx-auto",
            div {
                class: "card bg-base-100 shadow-xl",
                div {
                    class: "card-body text-center",
                    h2 {
                        class: "card-title text-2xl mb-4 justify-center",
                        "Exercise Library"
                    }
                    p {
                        class: "text-base-content/70",
                        "Library view coming in Phase 5..."
                    }
                }
            }
        }
    }
}
