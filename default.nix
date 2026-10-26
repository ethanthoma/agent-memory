{
  pkgs ? import <nixpkgs> { },
}:
pkgs.rustPlatform.buildRustPackage {
  pname = "agent-memory";
  version = "0.1.0";
  src = pkgs.lib.fileset.toSource {
    root = ./.;
    fileset = pkgs.lib.fileset.unions [
      ./Cargo.toml
      ./Cargo.lock
      ./src
      ./tests
      ./evaluation
    ];
  };
  cargoLock.lockFile = ./Cargo.lock;
  RUSTFLAGS = "-D warnings";
  nativeBuildInputs = [
    pkgs.makeWrapper
    pkgs.git
  ];
  postCheck = ''
    cargo run --offline --release --target ${pkgs.stdenv.hostPlatform.rust.rustcTarget} --bin agent-memory-eval -- evaluation
  '';
  postInstall = ''
    wrapProgram $out/bin/agent-memory --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.git ]}
  '';
}
