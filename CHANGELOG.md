# Changelog

## [0.3.0](https://github.com/corploc/kmlcli/compare/v0.2.0...v0.3.0) (2026-05-25)


### Features

* **parser:** resolve Kml::StyleMap to its normal style ([087c8d2](https://github.com/corploc/kmlcli/commit/087c8d2d55ff7fe7df941160735a1e94f8038151))


### Bug Fixes

* **projection:** clamp center_lat to ±85.05 in pan_up/pan_down ([acb4bcc](https://github.com/corploc/kmlcli/commit/acb4bccaa3fcda040ca223013f834c10254667ae))
* remove debug perf log left from development ([8429fef](https://github.com/corploc/kmlcli/commit/8429fefa47d00ee16620b3a10664d5263320e48b))
* **tiles:** cap prefetch list at MAX_VISIBLE_TILES (16) ([06e39f1](https://github.com/corploc/kmlcli/commit/06e39f18602eccbe0acd03ab95fc134a5f51fc8f))
* **tiles:** clamp latitude in ll2tile to Mercator valid range ([65a02e1](https://github.com/corploc/kmlcli/commit/65a02e1628258a72639131dd40e1020c0573355c))
* **tiles:** propagate HTTP client build errors instead of panicking ([4cf4196](https://github.com/corploc/kmlcli/commit/4cf419625411c53a5523ffe2809b7cddcda5c474))
* **tiles:** sleep 50ms instead of busy-spinning on missing URL template ([071e1f8](https://github.com/corploc/kmlcli/commit/071e1f81c5ed77a0931127fb92eabb1f88a35ff3))
* **tiles:** use CARGO_PKG_VERSION in HTTP user-agent ([b133d90](https://github.com/corploc/kmlcli/commit/b133d9082d6a68726987168cf7990d0201c5f760))
* **tui:** install panic hook before constructing App ([eda82ee](https://github.com/corploc/kmlcli/commit/eda82ee64302ef7ee68d36df939d32647e5ae037))
* **tui:** use dynamic tree panel height for scroll math ([c1ca68d](https://github.com/corploc/kmlcli/commit/c1ca68d0cf8c189953cd059a5b166ee6271365a8))

## [0.2.0](https://github.com/corploc/kmlcli/compare/v0.1.1...v0.2.0) (2026-05-25)


### Features

* shell completions subcommand + declare MSRV 1.85 ([3b92217](https://github.com/corploc/kmlcli/commit/3b922170b7fc054e2c7572be679f169db000480e))

## [0.1.1](https://github.com/corploc/kmlcli/compare/v0.1.0...v0.1.1) (2026-05-25)


### Bug Fixes

* ctrl+scroll for zoom (not ctrl+shift) ([66af422](https://github.com/corploc/kmlcli/commit/66af422936b9ecef369acfcf3c5847688c3666b1))
* **deps:** use rustls instead of openssl for cross-compile compatibility ([829a4d6](https://github.com/corploc/kmlcli/commit/829a4d6de7befc5e256fc16ee19210fb4e0764cc))
