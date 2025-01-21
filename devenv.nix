{ pkgs, lib, config, inputs, ... }:

{
  packages = with pkgs; [ git openssl.dev ];
  languages.rust.enable = true;
  # See full reference at https://devenv.sh/reference/options/
}
