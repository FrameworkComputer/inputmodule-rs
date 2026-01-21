{ pkgs, lib, config, inputs, ... }:

{
  packages = with pkgs; [
    flip-link
    cargo-make
    pkg-config
    systemd
  ];

  languages.rust = {
    enable = true;
    targets = [ "thumbv6m-none-eabi" ];
    # https://devenv.sh/reference/options/#languagesrustchannel
    channel = "stable";
  };
}
