# Example configuration to add Linux Distribution Agent to your home.nix
# You can add this to your existing ~/.config/home-manager/home.nix

{ config, pkgs, ... }:

let
  # Import your local linux-distro-agent flake
  linux-distro-agent = (import /home/alex/linux-distro-agent).packages.${pkgs.system}.default;
in
{
  # Add the package to your environment
  home.packages = with pkgs; [
    # ... your existing packages ...
    linux-distro-agent
  ];

  # Add useful aliases for the tool
  home.shellAliases = {
    # ... your existing aliases ...
    
    # Linux Distribution Agent shortcuts
    "lda" = "linux-distro-agent";
    "detect" = "linux-distro-agent detect";
    "install" = "linux-distro-agent install";
    "search" = "linux-distro-agent search";
    "update" = "linux-distro-agent update";
    "remove" = "linux-distro-agent remove";
    "list-distros" = "linux-distro-agent list-supported";
    "sys-info" = "linux-distro-agent info --pretty";
  };

  # Enable shell completions (automatically handled by the flake)
  programs.zsh.initExtra = ''
    # Linux Distribution Agent completions
    source <(linux-distro-agent completions zsh)
  '';
}
