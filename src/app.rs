use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center min-h-screen bg-white",
            div {
                class: "text-center space-y-6 px-6",
                h1 {
                    class: "text-4xl font-bold text-gray-900",
                    "Simple Strength Assistant"
                }
                p {
                    class: "text-gray-600",
                    "Tailwind CSS + DaisyUI Integration Test"
                }
                div {
                    class: "space-x-4",
                    button {
                        class: "btn btn-primary",
                        "Primary Button"
                    }
                    button {
                        class: "btn btn-secondary",
                        "Secondary Button"
                    }
                }
                div {
                    class: "flex gap-4 mt-8",
                    div {
                        class: "card bg-base-100 shadow-xl w-64",
                        div {
                            class: "card-body",
                            h2 {
                                class: "card-title",
                                "Test Card"
                            }
                            p {
                                "This is a DaisyUI card component with Tailwind utilities."
                            }
                        }
                    }
                }
            }
        }
    }
}
