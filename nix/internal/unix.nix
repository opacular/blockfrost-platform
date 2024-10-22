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

  cardano-node-flake = let
    unpatched = inputs.cardano-node;
  in
    (import inputs.flake-compat {
      src =
        if targetSystem != "aarch64-darwin"
        then unpatched
        else {
          outPath = toString (pkgs.runCommand "source" {} ''
            cp -r ${unpatched} $out
            chmod -R +w $out
            cd $out
            echo ${lib.escapeShellArg (builtins.toJSON [targetSystem])} $out/nix/supported-systems.nix
          '');
          inherit (unpatched) rev shortRev lastModified lastModifiedDate;
        };
    })
    .defaultNix;

  cardano-node-packages =
    {
      x86_64-linux = cardano-node-flake.hydraJobs.x86_64-linux.musl;
      x86_64-darwin = cardano-node-flake.packages.x86_64-darwin;
      aarch64-darwin = cardano-node-flake.packages.aarch64-darwin;
    }
    .${targetSystem};

  inherit (cardano-node-packages) cardano-node cardano-cli;

  cardano-node-configs = builtins.path {
    name = "cardano-playground-configs";
    path = inputs.cardano-playground + "/static/book.play.dev.cardano.org/environments";
  };

  runNode = network: let
    stateDir =
      if pkgs.stdenv.isDarwin
      then "Library/Application Support/blockfrost-platform/${network}"
      else ".local/share/blockfrost-platform/${network}";
  in
    pkgs.writeShellScriptBin "run-node-${network}" ''
      stateDir="$HOME"/${lib.escapeShellArg stateDir}
      mkdir -p "$stateDir"
      set -x
      exec ${lib.getExe cardano-node} run \
        --config ${cardano-node-configs}/${network}/config.json \
        --topology ${cardano-node-configs}/${network}/topology.json \
        --socket-path "$stateDir"/node.socket \
        --database-path "$stateDir"/chain
    ''
    // {meta.description = "Runs cardano-node on ${network}";};
}
