{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let pkgs = nixpkgs.legacyPackages.${system}; in
        {
          devShells.default = import ./shell.nix { inherit pkgs; };

          packages.default = pkgs.rustPlatform.buildRustPackage rec {
            pname = "adthand";
            version = "1";
            src = ./.;
            cargoBuildFlags = [ "--workspace" ];

            useFetchCargoVendor = true;
            cargoHash = "sha256-N1bj1vqCg0NHVi+EBKPMhAlW++spVEuo2+oTDV3PbiE=";

            nativeBuildInputs = [ pkgs.pkg-config ];

            buildInputs = with pkgs;[
              alsa-lib
              openssl
              xdotool
            ];
          };

        }
      );
}
