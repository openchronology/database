{ pkgs ? import <nixpkgs> {} }:
(pkgs.buildFHSUserEnv {
  name = "pipzone";
  targetPkgs = pkgs: (with pkgs; [
    python311
    python311Packages.pip
    python311Packages.virtualenv
    postgresql_15
  ]);
  runScript = "
    python -m venv .venv
    source .venv/bin/activate
    pip install -r requirements.txt
    pgxn install pgmp
  ";
}).env
