{
  description = "Trinity, a program to visualise and interact with matrices";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-parts.url = "github:hercules-ci/flake-parts";

    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.pre-commit-hooks.flakeModule
      ];

      systems = ["x86_64-linux" "aarch64-linux"];
      perSystem = {
        config,
        system,
        ...
      }: let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [(import inputs.rust-overlay)];
        };

        # rustToolchainStable = pkgs.rust-bin.stable.latest.default;
        rustToolchainNightlyWith = extra: pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override extra);

        rustToolchain = rustToolchainNightlyWith {};

        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        commonArgs = {
          inherit src;
          strictDeps = true;
          buildInputs = bevyDeps;
          nativeBuildInputs = bevyDeps;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        bevyDeps = with pkgs; [
          pkg-config
          alsa-lib
          udev
        ];

        nativeBevyDeps =
          bevyDeps
          ++ (with pkgs; [
            # libGL
            libxkbcommon
            # mesa
            vulkan-loader
            # vulkan-validation-layers
            xorg.libX11
            # xorg.libxcb
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
            wayland
          ]);
      in rec {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs =
            [
              (rustToolchainNightlyWith {
                targets = ["wasm32-unknown-unknown"];
                extensions = ["rust-analyzer" "rust-src" "rust-std"];
              })
            ]
            ++ nativeBevyDeps
            ++ (with pkgs; [
              cargo-fuzz
              cargo-mutants
              cargo-nextest
              cargo-tarpaulin
              fd
              just
            ]);
          shellHook = ''
            ${config.pre-commit.installationScript}
            export RUST_BACKTRACE=1
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath nativeBevyDeps}:$LD_LIBRARY_PATH"
          '';
        };

        # See https://flake.parts/options/pre-commit-hooks-nix and
        # https://github.com/cachix/git-hooks.nix/blob/master/modules/hooks.nix
        # for all the available hooks and options
        pre-commit.settings.hooks = {
          check-added-large-files.enable = true;
          check-merge-conflicts.enable = true;
          check-toml.enable = true;
          check-vcs-permalinks.enable = true;
          check-yaml.enable = true;
          end-of-file-fixer.enable = true;
          trim-trailing-whitespace.enable = true;

          rustfmt = {
            enable = true;
            packageOverrides = {
              cargo = rustToolchain;
              rustfmt = rustToolchain;
            };
          };
        };

        checks =
          packages
          // {
            clippy = craneLib.cargoClippy (commonArgs
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              });

            fmt = craneLib.cargoFmt {
              inherit src;
            };

            doctests = craneLib.cargoTest (commonArgs
              // {
                inherit cargoArtifacts;
                cargoTestArgs = "--doc";
              });

            nextest = craneLib.cargoNextest (commonArgs
              // {
                inherit cargoArtifacts;
                partitions = 1;
                partitionType = "count";
                cargoNextestExtraArgs = "--no-fail-fast";
              });
          };

        packages = {
          trinity-native = let
            baseBin = craneLib.buildPackage (commonArgs
              // {
                pname = "trinity-native-base";
                inherit cargoArtifacts;
                inherit (craneLib.crateNameFromCargoToml {inherit src;}) version;
                nativeBuildInputs = nativeBevyDeps;
              });
          in
            pkgs.stdenv.mkDerivation {
              pname = "trinity-native";
              inherit (craneLib.crateNameFromCargoToml {inherit src;}) version;

              dontUnpack = true;
              dontBuild = true;

              nativeBuildInputs = [pkgs.makeWrapper];

              installPhase = ''
                mkdir -p $out/bin
                cp ${baseBin}/bin/trinity $out/bin/trinity
                wrapProgram $out/bin/trinity --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath nativeBevyDeps}"
              '';

              meta.mainProgram = "trinity";
            };

          doc = craneLib.cargoDoc (commonArgs
            // {
              inherit cargoArtifacts;
              cargoDocExtraArgs = "--no-deps --document-private-items";
              RUSTDOCFLAGS = "--deny warnings";
            });

          doc-with-deps = craneLib.cargoDoc (commonArgs
            // {
              inherit cargoArtifacts;
              cargoDocExtraArgs = "--document-private-items";
              RUSTDOCFLAGS = "--deny warnings";
            });
        };
      };
    };
}
