mod app;
mod models;

use app::App;

fn main() {
    dioxus::launch(App);
}
