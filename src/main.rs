mod app;
mod models;
mod state;

use app::App;

fn main() {
    dioxus::launch(App);
}
