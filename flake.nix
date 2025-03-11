{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
  }: let
    forEachSystem = f:
      nixpkgs.lib.genAttrs [
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
        "x86_64-linux"
      ] (system:
        f {
          inherit system;
          pkgs = import nixpkgs {inherit system;};
          fenix = fenix.packages.${system};
        });
  in {
    devShells = forEachSystem ({
      pkgs,
      system,
      fenix,
    }: {
      default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          (fenix.complete.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
            "miri"
          ])
          fenix.rust-analyzer
          (writeShellScriptBin "lldb-dap" ''
            ${pkgs.lib.getExe' pkgs.lldb "lldb-dap"} --pre-init-command  "command script import ${pkgs.fetchFromGitHub {
              owner = "cmrschwarz";
              repo = "rust-prettifier-for-lldb";
              rev = "v0.4";
              hash = "sha256-eje+Bs7kS87x9zCwH+7Tl1S/Bdv8dGkA0BoijOOdmeI=";
            }}/rust_prettifier_for_lldb.py" $@
          '')
        ];
      };
    });
  };
}
