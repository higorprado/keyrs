{ lib
, stdenv
, rustPlatform
, pkg-config
, wayland
, systemd
, libxkbcommon
}:

rustPlatform.buildRustPackage rec {
  pname = "keyrs";
  version = "0.2.1";

  src = lib.cleanSource ../.;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    wayland
    systemd
    libxkbcommon
  ];

  cargoBuildFlags = [
    "--features"
    "pure-rust"
    "--bins"
  ];

  # Integration tests depend on local input devices and compositor state.
  doCheck = false;

  postInstall = ''
    mkdir -p "$out/share/keyrs"
    cp -r ${../config.d.example} "$out/share/keyrs/config.d.example"
    cp -r ${../profiles} "$out/share/keyrs/profiles"
    cp ${../dist/keyrs-udev.rules} "$out/share/keyrs/keyrs-udev.rules"

    # Keep legacy runtime layout expected by keyrs-service.sh.
    ln -s "$out/share/keyrs/config.d.example" "$out/config.d.example"
    ln -s "$out/share/keyrs/profiles" "$out/profiles"
    mkdir -p "$out/dist"
    cp "$out/share/keyrs/keyrs-udev.rules" "$out/dist/keyrs-udev.rules"

    # Install runtime helper in read-only/runtime mode for Nix installs.
    install -m 755 ${../scripts/keyrs-service.sh} "$out/bin/.keyrs-service-real"
    cat > "$out/bin/keyrs-service" <<EOF
#!${stdenv.shell}
export KEYRS_RUNTIME_ONLY=1
exec "\$(dirname "\$0")/.keyrs-service-real" "\$@"
EOF
    chmod +x "$out/bin/keyrs-service"
  '';

  meta = with lib; {
    description = "Rust-based keyboard remapper for Wayland";
    license = licenses.mit;
    platforms = platforms.linux;
    mainProgram = "keyrs";
  };
}
