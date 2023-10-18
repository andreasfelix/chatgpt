{
  inputs = {
    systems.url = "github:nix-systems/default";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, systems, nixpkgs, crane }:
    let
      eachSystem = nixpkgs.lib.genAttrs (import systems);
      deps = system:
        let
          pkgs = import nixpkgs { inherit system; };
          lib = pkgs.lib;
          craneLib = crane.lib.${system};

          src = craneLib.cleanCargoSource (craneLib.path ./.);
          commonArgs = {
            inherit src;
            strictDeps = true;
            buildInputs = [ ]
              ++ lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];
          };
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          chatgpt = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });
          devShell = craneLib.devShell {
            packages = with pkgs; [ rust-analyzer ];
            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          };
        in
        {
          inherit chatgpt devShell craneLib;
        };
    in
    {
      packages = eachSystem (system: with deps system; {
        default = chatgpt;
      });
      devShells = eachSystem (system: with deps system; {
        default = devShell;
      });
    };
}
