{ inputs, pkgs, lib, config, ... }:
  let
          #links libraries to shell
    libPath = pkgs.lib.makeLibraryPath [
            pkgs-stable.git
            pkgs.clang
            pkgs.opencv
            pkgs.libclang    
            pkgs.libglvnd
            pkgs.wayland
            pkgs.glxinfo
            pkgs.libxkbcommon
          ];
  pkgs-stable = import inputs.nixpkgs-stable { system = pkgs.stdenv.system; };
  in
{
  languages = {
        rust = { enable = true; };
        c.enable = true;
        };
  packages = [
    pkgs-stable.git
    pkgs.clang
    pkgs.opencv
    pkgs.libclang    
    pkgs.libglvnd
    pkgs.wayland
    pkgs.glxinfo
    pkgs.libxkbcommon
  ];

  git-hooks.hooks.clippy.enable = true;
  git-hooks.hooks.rustfmt.enable = true;
  env.LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
  env.LD_LIBRARY_PATH = libPath;
  enterShell = " glxinfo -B; ";
}
