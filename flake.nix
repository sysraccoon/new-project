{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
      };
      project-package = pkgs.callPackage ./package.nix {};
      project-name = project-package.name;
    in {
      packages = {
        ${project-name} = project-package;
        default = project-package;
      };

      apps = {
        ${project-name} = flake-utils.lib.mkApp {
          drv = self.packages.${system}.${project-name};
        };
        default = self.apps.${system}.${project-name};
      };

      devShells.default = pkgs.mkShell {
        inputsFrom = [
          project-package
        ];

        buildInputs = (with pkgs; [
          cargo
          rustc
        ]);
      };
    });
}

