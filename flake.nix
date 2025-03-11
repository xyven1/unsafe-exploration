{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs = {
    self,
    nixpkgs,
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
        });
  in {
    devShells = forEachSystem ({
      pkgs,
      system,
    }: {
      default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          rustc
          cargo
          cargo-wizard
          rust-analyzer
          clippy
          rustfmt
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
