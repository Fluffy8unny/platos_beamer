{ pkgs, lib, config, ... }:
{
  languages = { 
        rust = { enable = true; }; 
        c.enable = true; 
        };

  packages = [
    pkgs.clang
    pkgs.opencv
    pkgs.libclang
    pkgs.xorg.xauth
    pkgs.xorg.libX11
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.libxkbcommon
    pkgs.libglvnd
  ];

  git-hooks.hooks.clippy.enable = true;
  git-hooks.hooks.rustfmt.enable = true;
  env.LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
  env.LD_LIBRARY_PATH="${pkgs.libglvnd}/lib:${pkgs.libxkbcommon}/lib:$LD_LIBRARY_PATH";
}
