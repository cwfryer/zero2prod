with import <nixpkgs> {}; let
  fenix =
    callPackage
    (fetchFromGitHub {
      owner = "nix-community";
      repo = "fenix";
      # commit from: 2023-03-03
      rev = "e2ea04982b892263c4d939f1cc3bf60a9c4deaa1";
      hash = "sha256-AsOim1A8KKtMWIxG+lXh5Q4P2bhOZjoUhFWJ1EuZNNk=";
    })
    {};
in
  mkShell {
    name = "rust-env";
    nativeBuildInputs = [
      # Note: to use stable, just replace `default` with `stable`
      fenix.default.toolchain

      # Example Build-time Additional Dependencies
      pkg-config
    ];
    buildInputs = [
      # Example Run-time Additional Dependencies
      openssl
      lld
    ];
    packages = with pkgs; [
      postgresql_15
      doctl
    ];
    shellHook = ''
      export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
      export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
      export DATABASE_URL=postgres://postgres:password@localhost:5432/newsletter
    '';
  }
