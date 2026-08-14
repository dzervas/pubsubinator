with import <nixpkgs> {}; let
  wlink = rustPlatform.buildRustPackage {
    pname = "wlink";
    version = "0.1.0";

    src = fetchFromGitHub {
      owner = "ch32-rs";
      repo = "wlink";
      rev = "master";
      sha256 = "sha256-fKbtLGD6ch9S8WP2UhUBFisYko6BKK8v9HaHstUAKoE=";
    };

    cargoHash = "sha256-eLRCixVszQb/oktLGG4fUbPHZU+RNzYVyQpnc6yqW1U=";

    nativeBuildInputs = [ pkg-config ];
    buildInputs = [ libudev-zero ];
  };
in mkShell {
  nativeBuildInputs = [
    rustup
    probe-rs-tools

    wch-isp
    wlink
  ];
}
