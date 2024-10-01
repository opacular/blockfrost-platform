{
  inputs,
  targetSystem,
}:
# For now, let's keep all UNIX definitions together, until they diverge more in the future.
assert __elem targetSystem ["x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin"]; let
  buildSystem = targetSystem;
  pkgs = inputs.nixpkgs.legacyPackages.${buildSystem};
  inherit (pkgs) lib;
in rec {
  craneLib = inputs.crane.mkLib pkgs;

  src = craneLib.cleanCargoSource ../../.;

  commonArgs = {
    inherit src;
    strictDeps = true;
    nativeBuildInputs = lib.optionals pkgs.stdenv.isLinux [
      pkgs.pkg-config
    ];
    buildInputs =
      lib.optionals pkgs.stdenv.isLinux [
        pkgs.openssl
      ]
      ++ lib.optionals pkgs.stdenv.isDarwin [
        pkgs.libiconv
        pkgs.darwin.apple_sdk_12_3.frameworks.SystemConfiguration
        pkgs.darwin.apple_sdk_12_3.frameworks.Security
        pkgs.darwin.apple_sdk_12_3.frameworks.CoreFoundation
      ];
  };

  # For better caching:
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  package = craneLib.buildPackage (commonArgs
    // {
      inherit cargoArtifacts;
    });
}
