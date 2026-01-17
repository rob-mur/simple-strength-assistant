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

  languages.javascript = {
    enable = true;
    npm = {
      enable = true;
      install.enable = true;
    };
  };

  scripts = {
    dev.exec = "trunk serve";
    build.exec = "trunk build --release";
    format.exec = "cargo fmt";
    lint.exec = "./scripts/lint.sh";
    test.exec = "cargo test";
  };

  git-hooks.hooks = {
    ci-checks = {
      enable = true;
      name = "Code quality checks (format, clippy, test, build)";
      entry = ''
        format
        lint
        test
        build
      '';
      stages = ["pre-commit" "pre-push"];
    };
  };
}
