# Changelog

## [0.9.1](https://github.com/jetsaredim/autographs/compare/v0.9.0...v0.9.1) (2026-11-11)


### Bug Fixes

* complete admin publishes beyond proxy timeout ([81c7633](https://github.com/jetsaredim/autographs/commit/81c7633c1ec53fa1346f5f5b33a6f6f5c5d69ce9))
* complete admin publishes beyond proxy timeout ([aabec41](https://github.com/jetsaredim/autographs/commit/aabec41c48bee625216a85fa050b935c0e590f7b))
* keep publish status tied to its request ([b235677](https://github.com/jetsaredim/autographs/commit/b235677acbc85e5a81e79c593a9a52ce0fb9e062))
* preserve publish operation identity and results ([b4e1d37](https://github.com/jetsaredim/autographs/commit/b4e1d374edcfe4a5db7dee7f585db6eaf7b16f54))
* terminalize publish operations on errors and aborts ([8079c32](https://github.com/jetsaredim/autographs/commit/8079c32de570b13dde3137ae0ef6bc7be4aa9c29))

## [0.9.0](https://github.com/jetsaredim/autographs/compare/v0.8.2...v0.9.0) (2026-11-10)


### Features

* **08-08:** prepare Cloudflare domain DNS Terraform ([f95ca2d](https://github.com/jetsaredim/autographs/commit/f95ca2de6e2a9556abc55ae1578d790ae9e2cc83))
* import existing Cloudflare zone into Terraform state ([cda64dc](https://github.com/jetsaredim/autographs/commit/cda64dccb21bb404eb311d5c7718ad9e2cadc9f7))
* **infra:** add guarded Cloudflare proxy cutover ([1d70658](https://github.com/jetsaredim/autographs/commit/1d7065870e79783f519d9ecb2fea2e12d0bdc064))
* **infra:** add guarded Cloudflare proxy cutover ([2ece010](https://github.com/jetsaredim/autographs/commit/2ece010bd2d783482905dfb014704e64a3bd9a0f))
* **infra:** add guarded Cloudflare settings apply ([a3bdec9](https://github.com/jetsaredim/autographs/commit/a3bdec9cc5fd5babd82dad4430757ad7aea38687))
* **infra:** add guarded Cloudflare settings apply ([659bc98](https://github.com/jetsaredim/autographs/commit/659bc98f4af05e8544e35fa66c6933f96a01875a))
* prepare Cloudflare DNS in runtime Terraform root ([39411cb](https://github.com/jetsaredim/autographs/commit/39411cbbc26e9cde7906db2afa042f585fa1c6b1))
* prepare controlled Cloudflare state import ([a7f9dfb](https://github.com/jetsaredim/autographs/commit/a7f9dfbf8265d61b3b9e06f27f20296e2e03e97b))


### Bug Fixes

* accept quoted SPF representation during import ([4efe474](https://github.com/jetsaredim/autographs/commit/4efe474da8394bf688eb4b121cff5dcb24c869f4))
* align Cloudflare proxy TTL with automatic mode ([cc01fa7](https://github.com/jetsaredim/autographs/commit/cc01fa7bd0059dbc76fa3c68bf79a42662d407ce))
* align imported Cloudflare SPF state ([458d8bf](https://github.com/jetsaredim/autographs/commit/458d8bfb5880039ccc11d5ef9e0f40ffd67f26fe))
* align Terraform with imported Cloudflare SPF ([42eaa5a](https://github.com/jetsaredim/autographs/commit/42eaa5a1635fb30a765249bc72efd65d6a442432))
* diagnose Cloudflare import inventory mismatch ([9b5aa2a](https://github.com/jetsaredim/autographs/commit/9b5aa2a9586a138919dfae4c26d7fa8ae97be20f))
* guard Cloudflare import state and current runbooks ([55dd9e1](https://github.com/jetsaredim/autographs/commit/55dd9e141e96bca98bb9e9ee9283f40433375563))
* **infra:** preserve all other DNS record fields at cutover ([9fb6e29](https://github.com/jetsaredim/autographs/commit/9fb6e29daf60a6a7ab1ea14377a5ebc3be147d00))
* **infra:** reject disabled or altered cache rules ([c615c82](https://github.com/jetsaredim/autographs/commit/c615c8227a2a9d8d0fa576eeb0e3c9f993a9b685))
* **infra:** verify Cloudflare plan semantics before apply ([b33e426](https://github.com/jetsaredim/autographs/commit/b33e4261473748256a0a152d7cdc582cd61d93a4))
* keep Cloudflare inventory diagnostics private ([304f5f5](https://github.com/jetsaredim/autographs/commit/304f5f5e4eb32d5cfc7ba00dd2214adc101b1a5b))
* normalize Cloudflare TXT quotes during import check ([aa0570e](https://github.com/jetsaredim/autographs/commit/aa0570e0485fd615634b8d202a38df16c4c9d575))
* report Cloudflare import inventory mismatch safely ([f76125a](https://github.com/jetsaredim/autographs/commit/f76125acb5e285c69b9c46836896dbae96b57709))


### Performance Improvements

* add correlated publish timing and media IO logs ([687546f](https://github.com/jetsaredim/autographs/commit/687546ff01a95d07e423506cb90faa6b7ab1d348))
* reduce repeated shared media verification during publish ([145efa0](https://github.com/jetsaredim/autographs/commit/145efa0efe0a32743a5c5854b1c6f6b5ce01a2bf))
* reduce repeated shared media verification during publish ([d0c8f1b](https://github.com/jetsaredim/autographs/commit/d0c8f1bc5d218d2803112fb2a9fbc50528b6d6e5))

## [0.8.2](https://github.com/jetsaredim/autographs/compare/v0.8.1...v0.8.2) (2026-11-05)


### Bug Fixes

* **admin:** close publish freeze review gaps ([38e6368](https://github.com/jetsaredim/autographs/commit/38e6368115b08f0daac29dfa79bca92f2810bcf1))
* **admin:** disable publishing during deployment freeze ([a6c5ed6](https://github.com/jetsaredim/autographs/commit/a6c5ed65823fc3c09c7967fba3ca3b55436dbed0))
* **admin:** disable publishing during deployment freeze ([3c9aee6](https://github.com/jetsaredim/autographs/commit/3c9aee6f9b892c5bb0321ef2e859287f18805b1e))

## [0.8.1](https://github.com/jetsaredim/autographs/compare/v0.8.0...v0.8.1) (2026-11-05)


### Bug Fixes

* **admin:** honor framing target and restore group border ([d305b8f](https://github.com/jetsaredim/autographs/commit/d305b8f39a761f43ce7a1c78e2a69f8efcd43a22))
* **admin:** honor framing target and restore group border ([13f5323](https://github.com/jetsaredim/autographs/commit/13f5323655b4e1ae43d21dd9a231101ba63613b8))

## [0.8.0](https://github.com/jetsaredim/autographs/compare/v0.7.0...v0.8.0) (2026-11-05)


### Features

* improve image review workbench layout ([296762d](https://github.com/jetsaredim/autographs/commit/296762d788537a4eb0dd33aebd2273694e7fb4e2))
* integrate image review workbench layout ([bb4f846](https://github.com/jetsaredim/autographs/commit/bb4f846ba58abc8c646ae4f5a53fd6200c30a0f0))


### Bug Fixes

* **08:** WR-01 align JSON highlights with dirty state ([d6837e5](https://github.com/jetsaredim/autographs/commit/d6837e55a24503657ba8dd6731c9b9bb79712b16))

## [0.7.0](https://github.com/jetsaredim/autographs/compare/v0.6.0...v0.7.0) (2026-11-04)


### Features

* **admin:** improve image review controls and preview layout ([36595a8](https://github.com/jetsaredim/autographs/commit/36595a84581656eb532e4366cda860a5f86bc5ae))
* **admin:** improve image review controls and preview layout ([162618f](https://github.com/jetsaredim/autographs/commit/162618f0eb331f8a692c5b337643cdfbe8425e66))


### Bug Fixes

* **admin:** preserve saved adjustment and compact review at tablet widths ([f63604f](https://github.com/jetsaredim/autographs/commit/f63604f408aeb21768dedd4809e13dc73174a8b1))

## [0.6.0](https://github.com/jetsaredim/autographs/compare/v0.5.0...v0.6.0) (2026-11-02)


### Features

* restore admin image alignment and framing workflow ([a2684d0](https://github.com/jetsaredim/autographs/commit/a2684d04a3464c8711f49bb5a07ce3a366104c12))
* restore admin image alignment and framing workflow ([3470e78](https://github.com/jetsaredim/autographs/commit/3470e7830f2a4c01d5096317885a682e5276f6ad))


### Bug Fixes

* accept current-proven media in legacy cutover ([f57b0eb](https://github.com/jetsaredim/autographs/commit/f57b0eb592c753c21f09af903a8ab1730b503561))
* align retry workflow contract with pinned helper ([ba293a1](https://github.com/jetsaredim/autographs/commit/ba293a124179a3e1aaa8026454703d74e8910415))
* allow verified v0.5.0 finalization after unfreeze ([a2b56b8](https://github.com/jetsaredim/autographs/commit/a2b56b83b7ac4468d5f6dd77b882cef88e2e50d3))
* automate publish freeze before controller deployment ([7e854fd](https://github.com/jetsaredim/autographs/commit/7e854fd939c6604fd1817ace770a1491fb604be4))
* automate publish freeze before controller deployment ([6fddb5c](https://github.com/jetsaredim/autographs/commit/6fddb5cdf58eae5cdf10a6657028df8c7422e0e8))
* bind v0.5.0 finalization to exact run attempt ([55aa293](https://github.com/jetsaredim/autographs/commit/55aa2936a581166791f965b0b66399a66ff9fd46))
* complete frozen v0.5.0 release with audited unfreeze ([3bf83a6](https://github.com/jetsaredim/autographs/commit/3bf83a6ad8e149d4bfab451d973e4455ca9ef129))
* **deps:** update rust crate oracledb to 26.0.0-beta.5 ([f36a001](https://github.com/jetsaredim/autographs/commit/f36a0019b945597957ac4975733b4f4f0d703898))
* **deps:** update rust crate oracledb to 26.0.0-beta.5 ([0a70b91](https://github.com/jetsaredim/autographs/commit/0a70b91b4a0b3f4941a7eaae8a26d19891309bd5))
* keep rejected framing boundaries editable ([44f4f2b](https://github.com/jetsaredim/autographs/commit/44f4f2bdf7c6679344f9c45c05ddc1add4d3094b))
* remove unaudited v0.5.0 retry override ([365c271](https://github.com/jetsaredim/autographs/commit/365c2717c263cb9f49792be651dff2c81830dc0a))
* validate camelCase public release manifest during unfreeze ([f4aeece](https://github.com/jetsaredim/autographs/commit/f4aeece80a2290849f15cc741efee87861db978f))
* validate image suggestions and keep output guides aligned ([0fc10fa](https://github.com/jetsaredim/autographs/commit/0fc10fa1997f816951159bc320da93d1de0ed636))
* verify private publish status after unfreeze restart ([b62ea9d](https://github.com/jetsaredim/autographs/commit/b62ea9dbb6e8d5a0ac91e42f2e765653c3eaf7d7))
* verify private publish status after unfreeze restart ([9df5930](https://github.com/jetsaredim/autographs/commit/9df5930cc2621fe6b2c3a515546bac1c86c4d9c6))
* verify private publish status during v0.5.0 unfreeze ([f383279](https://github.com/jetsaredim/autographs/commit/f383279146866e3a35c8b74823b11d966a4ec6d6))
* verify private publish status during v0.5.0 unfreeze ([b09c7b0](https://github.com/jetsaredim/autographs/commit/b09c7b0be43ada4172bbdda76e1a48bd552a3949))

## [0.5.0](https://github.com/jetsaredim/autographs/compare/v0.4.2...v0.5.0) (2026-10-31)


### Features

* improve automatic image framing detection ([#281](https://github.com/jetsaredim/autographs/issues/281)) ([561c53e](https://github.com/jetsaredim/autographs/commit/561c53e5e723bfbdddbbbd50a47f935dc167347a))
* publish adjusted media with CDN-ready URLs ([#294](https://github.com/jetsaredim/autographs/issues/294)) ([43bdd29](https://github.com/jetsaredim/autographs/commit/43bdd290289ba1104d33bd931fa6b9757888ce3b))


### Bug Fixes

* **deps:** update routine dependency updates ([#283](https://github.com/jetsaredim/autographs/issues/283)) ([0717158](https://github.com/jetsaredim/autographs/commit/0717158cfb99851e10e46f4db74a8a6a6d0c0f0c))
* expose security classification evidence for issue 291 ([#292](https://github.com/jetsaredim/autographs/issues/292)) ([27babb4](https://github.com/jetsaredim/autographs/commit/27babb46a79827afc3b175af049a782a61526a9e))
* **renovate:** keep Terraform CLI updates atomic ([#287](https://github.com/jetsaredim/autographs/issues/287)) ([1123235](https://github.com/jetsaredim/autographs/commit/1123235bf8ce51f2aaa87d0827bda642ca80e85a))
* **tests:** avoid managed dependency version assertions ([#290](https://github.com/jetsaredim/autographs/issues/290)) ([2fe6333](https://github.com/jetsaredim/autographs/commit/2fe63333a94b946f9164b3dd255bc7916ff3d071))


### Performance Improvements

* **ci:** restore Ansible syntax and lint validation ([#300](https://github.com/jetsaredim/autographs/issues/300)) ([bb2b783](https://github.com/jetsaredim/autographs/commit/bb2b7830cc621ad32a4ace611222cba309d5bc1c))

## [0.4.2](https://github.com/jetsaredim/autographs/compare/v0.4.1...v0.4.2) (2026-10-15)


### Miscellaneous Chores

* **controller:** extract shared image processing crate ([#279](https://github.com/jetsaredim/autographs/issues/279)) ([38fa9c3](https://github.com/jetsaredim/autographs/commit/38fa9c3e706562c00889b1ddb1ff6ed89b02b05a))

## [0.4.1](https://github.com/jetsaredim/autographs/compare/v0.4.0...v0.4.1) (2026-10-15)


### Bug Fixes

* isolate image review state and overlays ([#276](https://github.com/jetsaredim/autographs/issues/276)) ([0f80f41](https://github.com/jetsaredim/autographs/commit/0f80f41143824c2a07b9cc3a44b1c2f7b81f5a00))

## [0.4.0](https://github.com/jetsaredim/autographs/compare/v0.3.0...v0.4.0) (2026-10-12)


### Features

* **admin:** add private image review workflow ([#263](https://github.com/jetsaredim/autographs/issues/263)) ([921a397](https://github.com/jetsaredim/autographs/commit/921a3977a3b17f051dfb8a199033363bb0863d90))
* **deps:** update oracledb crate ([#270](https://github.com/jetsaredim/autographs/issues/270)) ([74d798b](https://github.com/jetsaredim/autographs/commit/74d798b618feed9bb8d9c98b862a08f76ca81d51))
* remove container provenance manifest generation ([#271](https://github.com/jetsaredim/autographs/issues/271)) ([1f203b0](https://github.com/jetsaredim/autographs/commit/1f203b0ca260ac5cc98edccd3fc9205b62b4b054))
* tighten ghcr image cleanup ([#272](https://github.com/jetsaredim/autographs/issues/272)) ([2f93620](https://github.com/jetsaredim/autographs/commit/2f93620e30b97c71a15ed04156934af806008c2f))


### Bug Fixes

* **deps:** update routine dependency updates ([#267](https://github.com/jetsaredim/autographs/issues/267)) ([d6e50dd](https://github.com/jetsaredim/autographs/commit/d6e50dd287aa7c6c9a2dc28879af81a7fa78b8a8))

## [0.3.0](https://github.com/jetsaredim/autographs/compare/v0.2.5...v0.3.0) (2026-10-01)


### Features

* persist Oracle image adjustments ([#261](https://github.com/jetsaredim/autographs/issues/261)) ([15c0779](https://github.com/jetsaredim/autographs/commit/15c07797723f6433699714a7471d6a3339ffea2b))

## [0.2.5](https://github.com/jetsaredim/autographs/compare/v0.2.4...v0.2.5) (2026-09-28)


### Bug Fixes

* **deploy:** handle absent kernel probe override ([#256](https://github.com/jetsaredim/autographs/issues/256)) ([75bb0a9](https://github.com/jetsaredim/autographs/commit/75bb0a9979639a32a7bb3d8da2507bc33f5c38f9))
* **deploy:** isolate retry status checkout ([#258](https://github.com/jetsaredim/autographs/issues/258)) ([98f4e8b](https://github.com/jetsaredim/autographs/commit/98f4e8b801435f8927a536cccc03c3ff8b53dc5c))
* **deploy:** recover drafts with current automation ([#257](https://github.com/jetsaredim/autographs/issues/257)) ([75a3793](https://github.com/jetsaredim/autographs/commit/75a3793523da76158cb52c78fc09f407699b0ee2))

## [0.2.4](https://github.com/jetsaredim/autographs/compare/v0.2.3...v0.2.4) (2026-09-26)


### Bug Fixes

* **patching:** enforce UEK-only kernel posture ([#253](https://github.com/jetsaredim/autographs/issues/253)) ([103a01f](https://github.com/jetsaredim/autographs/commit/103a01fa8676b590bbc058090e58edb433339981))

## [0.2.3](https://github.com/jetsaredim/autographs/compare/v0.2.2...v0.2.3) (2026-09-20)


### Bug Fixes

* **admin:** align taxonomy and signer cells ([#250](https://github.com/jetsaredim/autographs/issues/250)) ([ae55973](https://github.com/jetsaredim/autographs/commit/ae559732d13ef4af65a8040b2fa56be254743642))

## [0.2.2](https://github.com/jetsaredim/autographs/compare/v0.2.1...v0.2.2) (2026-09-19)


### Bug Fixes

* **deps:** update routine dependency updates ([#245](https://github.com/jetsaredim/autographs/issues/245)) ([517d1d5](https://github.com/jetsaredim/autographs/commit/517d1d5af6542953ae050019d1092b22c9d4cafa))
* reconcile security patch issue state ([#247](https://github.com/jetsaredim/autographs/issues/247)) ([82cf104](https://github.com/jetsaredim/autographs/commit/82cf104a2e23bd99defe9ad4cf789e5c05180ce6))


### Performance Improvements

* **ci:** streamline controller image builds ([#249](https://github.com/jetsaredim/autographs/issues/249)) ([513c732](https://github.com/jetsaredim/autographs/commit/513c7326d282299c97b97fbf72fb764ae1348bf0))

## [0.2.1](https://github.com/jetsaredim/autographs/compare/v0.2.0...v0.2.1) (2026-09-07)


### Bug Fixes

* **release:** fail closed before publication ([1bd2df3](https://github.com/jetsaredim/autographs/commit/1bd2df30fa24d60ce680c92e224114a01c93c40a))
* **release:** fail closed before publication ([c0c7bce](https://github.com/jetsaredim/autographs/commit/c0c7bce8bf33727e567d4ca784edc4e7c9030f59))
* **release:** resolve draft manifest assets ([d7ccd31](https://github.com/jetsaredim/autographs/commit/d7ccd31cdc886ef489af23a5548bec16e768c1ce))
* **release:** resolve draft manifest assets ([dd77090](https://github.com/jetsaredim/autographs/commit/dd77090fbf74c1e022df1b6cc2e6f18c63bddcbc))

## [0.2.0](https://github.com/jetsaredim/autographs/compare/v0.1.8...v0.2.0) (2026-09-06)


### Features

* **releases:** gate production deployment on release-please ([7ad73dd](https://github.com/jetsaredim/autographs/commit/7ad73ddb1a52683881bded5c1a7d55fc9ef67507))


### Bug Fixes

* **release:** close retry and dispatch race gaps ([4618b65](https://github.com/jetsaredim/autographs/commit/4618b6585a3b80bead251202ee25d27f40e45acc))
* **release:** reject stale reused-controller retries ([636827f](https://github.com/jetsaredim/autographs/commit/636827fa94ed7cec35737a79fe8105dad50e1d8d))
* **release:** serialize release automation and authenticate rollback ([7f91219](https://github.com/jetsaredim/autographs/commit/7f91219135e8de629ccda552ad0511a38f20a050))
* **releases:** fail closed on registry probe errors ([1370d54](https://github.com/jetsaredim/autographs/commit/1370d54d5559af14da1588bb825bfd27ca25bd2c))
* **releases:** mint short-lived GitHub App token ([82c69a9](https://github.com/jetsaredim/autographs/commit/82c69a9e0c720b1513551dd3895780198021c52b))
* **release:** surface invalid dispatch refs ([4791fb1](https://github.com/jetsaredim/autographs/commit/4791fb16d89acae2c70191ce613c809035130963))

## Changelog

Release notes are maintained by the release-please Release PR.
