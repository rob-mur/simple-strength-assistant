mod app;
mod service_worker;

use app::App;

fn main() {
    dioxus::launch(App);
}
