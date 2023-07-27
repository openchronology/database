{ pkgs ? import <nixpkgs> {} }:
let
  my-python-packages = ps: with ps; [
    (
      buildPythonPackage rec {
        pname = "pgxnclient";
        version = "1.3.2";
        src = fetchPypi {
          inherit pname version;
          sha256 = "b0343e044b8d0044ff4be585ecce0147b1007db7ae8b12743bf222758a4ec7d9";
        };
        doCheck = false;
        propagatedBuildInputs = [
        ];
      }
    )
  ];
in
pkgs.mkShell {
  # nativeBuildInputs is usually what you want -- tools you need to run
  nativeBuildInputs = with pkgs.buildPackages; [ postgresql_15 (python3.withPackages my-python-packages) ];
}
