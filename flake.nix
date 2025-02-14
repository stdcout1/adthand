{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = with pkgs;[
            alsa-lib
            openssl
            xdotool
          ];
        in
        {
          devShells.default = pkgs.mkShell
            {
              inherit buildInputs nativeBuildInputs;
            };
          packages.default = pkgs.rustPlatform.buildRustPackage rec {
            inherit buildInputs nativeBuildInputs;
            pname = "adthand";
            version = "1";
            src = ./.;
            cargoBuildFlags = [ "--workspace" ];

            useFetchCargoVendor = true;
            cargoHash = "sha256-Tb4rG5A3KZ8gXUlE3/vgoA9nibURKOr/TwE/ddIh1eM=";

            postFixup = ''
              mv $out/bin/dameon $out/bin/adthand-dameon
            '';
          };

        }
      );
}
