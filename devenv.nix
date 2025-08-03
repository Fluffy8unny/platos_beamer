{ pkgs, lib, config, ... }:
{
  languages = { 
        rust = { enable = true; }; 
        c.enable = true; 
        };

  packages = [
    pkgs.clang
    pkgs.gtk3
    (pkgs.opencv.override{ enableGtk3 = true; })
    pkgs.libclang
  ];

  git-hooks.hooks.clippy.enable = true;
  git-hooks.hooks.rustfmt.enable = true;
  env.LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
}
