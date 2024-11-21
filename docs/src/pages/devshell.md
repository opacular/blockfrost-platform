## Devshell

This repository has a [devshell](https://github.com/numtide/devshell) configured for Linux and macOS machines, both x86-64, and AArch64. To use it, please:

1. Install:
   - [Nix](https://nixos.org/download/),
   - [direnv](https://direnv.net/),
   - optionally: [nix-direnv](https://github.com/nix-community/nix-direnv) for a slightly better performance, if it’s easy for you to enable, e.g. on NixOS, [nix-darwin](https://github.com/LnL7/nix-darwin), using [home-manager](https://github.com/nix-community/home-manager) etc.
2. Enter the cloned directory.
3. And run `direnv allow`.

### Pure Nix builds

You can also use `nix build` to build the package for these platforms.

If in doubt, run `nix flake show`.
