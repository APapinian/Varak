{
  description = "Varak: rnote fork tuned for the Violet desktop (NixOS, Qtile/Wayland, Zen 4)";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      lib = pkgs.lib;

      runtimeDeps = with pkgs; [
        appstream
        glib
        gst_all_1.gstreamer
        gtk4
        libadwaita
        libxml2
        alsa-lib
      ];

      buildTools = with pkgs; [
        appstream-glib
        cmake
        desktop-file-utils
        meson
        ninja
        pkg-config
        python3
        shared-mime-info
        gettext
      ];

      # Zen 4 (Ryzen 7 8840HS). Set `portable = true` for a generic x86-64-v3 build.
      mkVarak =
        {
          portable ? false,
        }:
        pkgs.stdenv.mkDerivation {
          pname = "varak";
          version = "0.15.0";
          src = lib.cleanSource ./.;

          cargoDeps = pkgs.rustPlatform.importCargoLock {
            lockFile = ./Cargo.lock;
            # Pinned so the git dependency is a shallow fixed-output fetch instead of a full clone.
            outputHashes = {
              "piet-0.8.0" = "sha256-wTALeI+wXkVb0ifx98RBg7iAa6jS54fFD+h+BTugGYk=";
            };
          };

          nativeBuildInputs = buildTools ++ [
            pkgs.rustPlatform.bindgenHook
            pkgs.rustPlatform.cargoSetupHook
            pkgs.cargo
            pkgs.rustc
            pkgs.wrapGAppsHook4
          ];
          buildInputs = runtimeDeps;

          dontUseCmakeConfigure = true;
          mesonFlags = [ (lib.mesonBool "cli" true) ];

          env.RUSTFLAGS = if portable then "-C target-cpu=x86-64-v3" else "-C target-cpu=znver4";

          postPatch = ''
            chmod +x build-aux/*.py
            patchShebangs build-aux
          '';

          meta = {
            description = "Handwritten notes, rnote fork for the Violet desktop";
            license = lib.licenses.gpl3Plus;
            platforms = [ system ];
            mainProgram = "rnote";
          };
        };
    in
    {
      packages.${system} = {
        default = mkVarak { };
        portable = mkVarak { portable = true; };
      };

      apps.${system}.default = {
        type = "app";
        program = lib.getExe self.packages.${system}.default;
      };

      devShells.${system}.default = pkgs.mkShell {
        packages =
          runtimeDeps
          ++ buildTools
          ++ (with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
            rust-analyzer
            rustPlatform.bindgenHook
            just
            mold
            perf
            cargo-flamegraph
            hyperfine
          ]);

        # Faster local iteration: mold linker + native CPU tuning.
        RUSTFLAGS = "-C target-cpu=native -C link-arg=-fuse-ld=mold";
        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
        # SVG icon loader (wrapGAppsHook does this for the package build).
        GDK_PIXBUF_MODULE_FILE = "${pkgs.librsvg}/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache";
        XDG_DATA_DIRS = lib.makeSearchPath "share/gsettings-schemas/${pkgs.gtk4.name}" [ pkgs.gtk4 ];
      };
    };
}
