{pkgs, ...}: {
  packages = with pkgs; [
    git
    spec-kit
    claude-code
    gh
    trunk
    wasm-bindgen-cli
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = ["wasm32-unknown-unknown"];
  };

  scripts = {
    dev.exec = "trunk serve";
    build.exec = "trunk build --release";
    format.exec = "cargo fmt";
    format-check.exec = "cargo fmt -- --check";
    lint.exec = "cargo clippy -- -D warnings";
    test.exec = "cargo test";
  };
}
