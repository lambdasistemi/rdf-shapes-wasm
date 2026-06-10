{ pkgs
, conformance
, src
}:

pkgs.runCommand "rdf-shapes-conformance-check"
  {
    nativeBuildInputs = [
      pkgs.apache-jena
      pkgs.glibcLocales
      pkgs.jdk
      pkgs.which
      conformance
    ];
    LANG = "C.UTF-8";
    LC_ALL = "C.UTF-8";
  }
  ''
    set -euo pipefail
    rdf-shapes-conformance ${src}/conformance/corpus
    touch "$out"
  ''
