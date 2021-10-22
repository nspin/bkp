with import <nixpkgs> {};

mkShell {
  nativeBuildInputs = [
    pkg-config
    openssl
    fuse
  ];
}
