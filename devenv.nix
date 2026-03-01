{pkgs, ...}: {
  devcontainer.enable = true;
  packages = with pkgs; [
    git
    gh
    dioxus-cli
    wasm-bindgen-cli
    binaryen
    devcontainer
    claude-code
    gemini-cli-bin
    chromium
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

  env = {
    CHROMIUM_EXECUTABLE_PATH = "${pkgs.chromium}/bin/chromium";
    PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = "1";
  };

  scripts = {
    build.exec = "dx bundle --web --release --debug-symbols=false";
    format.exec = "cargo fmt";
    lint.exec = "./scripts/lint.sh";
    ci-test.exec = "./scripts/ci-test.sh";
  };

  processes = {
    serve.exec = "dx serve --port 8080";
  };

  git-hooks.hooks = {
    ci-checks = {
      enable = true;
      name = "Code quality checks (format, clippy, test, build)";
      entry = ''
        format
        lint
        ci-test
        build
      '';
      stages = ["pre-commit" "pre-push"];
    };
  };
}
