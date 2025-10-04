{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      systems = [ "aarch64-darwin" "x86_64-darwin" "aarch64-linux" "x86_64-linux" ];
    in
    {
      formatter.x86_64-linux = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;
      devShells = nixpkgs.lib.genAttrs systems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default =
            pkgs.mkShell rec {
              nativeBuildInputs = with pkgs; [
                pkgs.rustup
                pkgs.vscode-extensions.vadimcn.vscode-lldb
                pkgs.openssl
                wayland
                stdenv.cc.cc
                xz
              ];
              VSCODE_CODELLDB = "${pkgs.vscode-extensions.vadimcn.vscode-lldb}";
              LD_LIBRARY_PATH = "/run/opengl-driver/lib:${ with pkgs; lib.makeLibraryPath
                nativeBuildInputs
              }";
              OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib";
              OPENSSL_DIR="${pkgs.openssl.dev}";
              RUSTFLAGS=''--cfg getrandom_backend="wasm_js"'';
              shellHook = ''
                export XDG_DATA_DIRS=$XDG_DATA_DIRS:${pkgs.gtk4}/share/gsettings-schemas/gtk4-4.16.3/
                export PATH=$PATH:${pkgs.vscode-extensions.vadimcn.vscode-lldb}/share/vscode/extensions/vadimcn.vscode-lldb/adapter/
                export PATH=$HOME/.cargo/bin:$PATH
                export CARGO_HOME="$HOME/.cargo"
              '';
            };
        });
    };
}
