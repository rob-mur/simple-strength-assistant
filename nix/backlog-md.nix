# Derivation for backlog.md CLI (https://github.com/MrLesk/backlog.md)
#
# backlog.md ships a pre-built native binary per platform via npm optional
# dependencies. This derivation fetches the linux-x64 binary tarball directly
# from the npm registry and patches it with autoPatchelfHook so it works on
# NixOS.
{
  pkgs ? import <nixpkgs> { },
}:

pkgs.stdenv.mkDerivation rec {
  pname = "backlog-md";
  version = "1.44.0";

  src = pkgs.fetchurl {
    url = "https://registry.npmjs.org/backlog.md-linux-x64/-/backlog.md-linux-x64-${version}.tgz";
    sha256 = "1zc2kfzpw5m80gjnjnp8x9hpqxrldghls1p8a64hiana9j7b45zy";
  };

  nativeBuildInputs = [ pkgs.autoPatchelfHook ];
  buildInputs = [
    pkgs.glibc
    pkgs.gcc-unwrapped.lib
    pkgs.zlib
  ];

  # The tarball unpacks to ./package/
  sourceRoot = "package";

  dontBuild = true;
  dontConfigure = true;

  # CRITICAL: Bun-compiled binaries append a zip payload containing the application code
  # to the end of the executable. The default Nix `strip` phase will remove this payload,
  # breaking the application and causing it to fall back to the default Bun REPL.
  dontStrip = true;

  installPhase = ''
    mkdir -p $out/bin
    install -Dm755 backlog $out/bin/backlog
  '';

  meta = with pkgs.lib; {
    description = "Backlog.md — git-native AI-friendly task management CLI";
    homepage = "https://github.com/MrLesk/backlog.md";
    license = licenses.mit;
    platforms = [ "x86_64-linux" ];
    mainProgram = "backlog";
  };
}
