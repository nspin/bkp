with import (builtins.fetchGit {
  url = "https://github.com/NixOS/nixpkgs.git";
  ref = "master";
  rev = "732717d4042b4062840868ef28ca7c19abfbfc0b";
}) {};

mkShell {
  nativeBuildInputs = [
    rustup
    pkg-config
    openssl
    fuse
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.Security
    libiconv
  ];
}
