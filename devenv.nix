{pkgs, ...}: {
  packages = with pkgs; [
    git
    gh
    dioxus-cli
    wasm-bindgen-cli
    binaryen
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
    build.exec = "dx bundle --web --release --debug-symbols=false && ./scripts/fix-vercel-config.sh";
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
