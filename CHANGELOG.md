# Changelog

Version v2.1.8
--------------------
Released: 2026-10-12

🚀 Features:
- *(ci)* Push packages to Gemfury on release
- *(release)* Add rpm and deb package artifacts
- *(container)* Update image docker.io/library/rust to v1.99.0

🐛 Bug Fixes:
- *(script)* Run script only on uninstall
- *(container)* Update image docker.io/library/alpine to v3.24.2

🚜 Refactor:
- Rename rpm package artifact

⚙️ Miscellaneous Tasks:
- Enforce conventional commits
- Add new changelog workflow
- *(builder)* Update image ghcr.io/heathcliff26/rust-builder to v202610090432

💼 Other:
- Fix coverprofiles permissions

Version v2.1.7
--------------------
Released: 2026-09-12

💼 Other:
- Allow unset variable on shell expansion

Version v2.1.6
--------------------
Released: 2026-09-12

🚀 Features:
- *(container)* [**breaking**] Update image ghcr.io/heathcliff26/rust-builder to v202609090334

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/rust to v1.98.1

💼 Other:
- Allow users to inject env

Version v2.1.5
--------------------
Released: 2026-09-07

🚀 Features:
- *(container)* [**breaking**] Update image ghcr.io/heathcliff26/rust-builder to v202608131539
- *(container)* Update image docker.io/library/rust to v1.98.0
- *(container)* [**breaking**] Update image ghcr.io/heathcliff26/rust-builder to v202609060502

💼 Other:
- Move to container repo
- Move helm to release workflow
- Add suffix to compressed package
- Switch logging to using target instead of envfilter

Version v2.1.4
--------------------
Released: 2026-08-12

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/rust to v1.97.1
- *(github-release)* Update goreleaser/goreleaser to v2.17.1

Version v2.1.3
--------------------
Released: 2026-07-12

🚀 Features:
- *(github-release)* Update goreleaser/goreleaser to v2.17.0
- *(container)* Update image docker.io/library/rust to v1.97.0

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/alpine to v3.24.1

Version v2.1.2
--------------------
Released: 2026-06-12

🚀 Features:
- *(container)* Update image docker.io/library/alpine to v3.24.0

💼 Other:
- Move renovate and Link Check to tekton
- Remove obsolete Renovate status badge

Version v2.1.1
--------------------
Released: 2026-06-01

🚀 Features:
- *(github-release)* Update dyne/musl to 2.2.0
- *(github-release)* Update goreleaser/goreleaser to v2.16.0
- *(container)* Update image docker.io/library/rust to v1.96.0

💼 Other:
- *(deps)* Bump openssl from 0.10.79 to 0.10.80

Version v2.1.0
--------------------
Released: 2026-05-15

💼 Other:
- *(deps)* Bump openssl from 0.10.78 to 0.10.79
- Add support for HTTPRoute and ListenerSet

Version v2.0.1
--------------------
Released: 2026-04-25

💼 Other:
- Add Commit sha to build args
- Fix permissions for coverprofile workflow
- Fix interval format
- *(deps)* Bump rustls-webpki from 0.103.12 to 0.103.13

Version v2.0.0
--------------------
Released: 2026-04-24

🚀 Features:
- *(container)* Update image docker.io/library/rust to v1.95.0

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/alpine to v3.23.4
- *(github-release)* Update goreleaser/goreleaser to v2.15.4

💼 Other:
- Update RPM build for rust version
- Update for new rust workflow
- Update manifest to drop metrics
- Update goreleaser to support rust cross compilation
- Update for v2
- Update excludes for publishing
- Do not publish to crates.io
- Update default memory request
- Add tests covering verify_token function
- *(deps)* Bump openssl from 0.10.77 to 0.10.78

Version v1.4.0
--------------------
Released: 2026-03-30

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/golang to v1.26.1
- *(deps)* Update module github.com/heathcliff26/simple-fileserver to v1.3.0

🛡️ Security:
- Use hardcoded security context

💼 Other:
- Use user and group nobody

Version v1.3.5
--------------------
Released: 2026-03-01

🚀 Features:
- *(container)* Update image docker.io/library/golang to v1.26.0

🐛 Bug Fixes:
- *(deps)* Update module github.com/heathcliff26/simple-fileserver to v1.2.18

Version v1.3.4
--------------------
Released: 2026-02-01

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/alpine to v3.23.3
- *(container)* Update image docker.io/library/golang to v1.25.6
- *(deps)* Update module github.com/heathcliff26/simple-fileserver to v1.2.17

💼 Other:
- Use go.yaml.in/yaml/v3 for YAML parsing
- Remove unneeded prepare step in renovate workflow

Version v1.3.3
--------------------
Released: 2026-01-01

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/alpine to v3.23.2
- *(deps)* Update module github.com/heathcliff26/simple-fileserver to v1.2.16

💼 Other:
- Set default resource requests
- Rollout pods when config changes

Version v1.3.2
--------------------
Released: 2025-12-11

🚀 Features:
- *(container)* Update image docker.io/library/alpine to v3.23.0

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/golang to v1.25.5
- *(deps)* Update module github.com/spf13/cobra to v1.10.2

Version v1.3.1
--------------------
Released: 2025-12-05

💼 Other:
- Relabel servicemonitor instance label

Version v1.3.0
--------------------
Released: 2025-12-05

💼 Other:
- Implement basic Prometheus metrics server
- Simplify update loop controls
- Add healthz endpoint
- Implement application specific metrics

Version v1.2.2
--------------------
Released: 2025-12-01

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/golang to v1.25.4
- *(deps)* Update module github.com/heathcliff26/simple-fileserver to v1.2.15

💼 Other:
- Enable signing artifacts in release workflow
- Consolidate targets after F41 moved to go1.24

Version v1.2.1
--------------------
Released: 2025-10-25

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/alpine to v3.22.2
- *(container)* Update image docker.io/library/golang to v1.25.3
- *(deps)* Update module github.com/heathcliff26/simple-fileserver to v1.2.13

💼 Other:
- Add Fedora 43 and Rawhide targets

Version v1.2.0
--------------------
Released: 2025-10-01

🐛 Bug Fixes:
- *(deps)* Update module github.com/spf13/cobra to v1.10.0
- *(deps)* Update module github.com/spf13/cobra to v1.10.1
- *(container)* Update image docker.io/library/golang to v1.25.1
- *(deps)* Update module github.com/heathcliff26/simple-fileserver to v1.2.11

💼 Other:
- Add binary leftover from packaing openwrt

Version v1.1.10
--------------------
Released: 2025-09-01

🚀 Features:
- *(container)* Update image docker.io/library/golang to v1.25.0

🐛 Bug Fixes:
- *(deps)* Update gomod

💼 Other:
- Switch to alpine as final base image

Version v1.1.9
--------------------
Released: 2025-08-01

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/golang to v1.24.5
- *(deps)* Update gomod

⚙️ Miscellaneous Tasks:
- *(container)* Update image docker.io/library/golang to 270cd53
- *(container)* Update image docker.io/library/golang to 1bb140b
- *(container)* Update image docker.io/library/golang to 1aa97dd
- *(container)* Update image docker.io/library/golang to a92f3b1
- *(container)* Update image docker.io/library/golang to 764d7e0
- *(container)* Update image docker.io/library/golang to 20a022e

Version v1.1.8
--------------------
Released: 2025-07-01

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/golang to v1.24.4
- *(deps)* Update gomod

Version v1.1.7
--------------------
Released: 2025-06-01

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/golang to v1.24.3
- *(deps)* Update module github.com/heathcliff26/simple-fileserver to v1.2.5

💼 Other:
- Tag release to ensure correct version in binary
- Update release job to target renamed workflow

Version v1.1.6
--------------------
Released: 2025-05-01

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/golang to v1.24.2
- *(deps)* Update module github.com/heathcliff26/simple-fileserver to v1.2.4

⚙️ Miscellaneous Tasks:
- *(container)* Update image docker.io/library/golang to 30baaea

💼 Other:
- Add help function
- Add gosec to improve code quality
- Move target description to above them
- Fix release workflow permissions

Version v1.1.5
--------------------
Released: 2025-04-01

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/golang to v1.24.1
- *(deps)* Update module github.com/heathcliff26/simple-fileserver to v1.2.3

💼 Other:
- Use middleware for logging and code simplification

Version v1.1.4
--------------------
Released: 2025-03-01

🐛 Bug Fixes:
- *(deps)* Update module github.com/spf13/cobra to v1.9.1

⚙️ Miscellaneous Tasks:
- *(container)* Update image docker.io/library/golang to 3f74443

💼 Other:
- Fix broken docker hub link
- Use the version from build info
- Replace abandoned yaml module with sig-kubernetes version

Version v1.1.3
--------------------
Released: 2025-02-01

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/golang to v1.23.5

💼 Other:
- Upload coverprofile to coveralls
- Add -trimpath option to minimally reduce binary size
- Fix missing explicit permissions for workflow runs

Version v1.1.2
--------------------
Released: 2025-01-01

🐛 Bug Fixes:
- *(deps)* Update module github.com/stretchr/testify to v1.10.0
- *(container)* Update image docker.io/library/golang to v1.23.3
- *(container)* Update image docker.io/library/golang to v1.23.4

⚙️ Miscellaneous Tasks:
- Tag container as latest by default
- *(container)* Update image docker.io/library/golang to e5ca199

💼 Other:
- Add fmt and validate commands
- Add validate step to ci job
- Update go version to 1.N.P notation

Version v1.1.1
--------------------
Released: 2024-11-01

🐛 Bug Fixes:
- *(container)* Update image docker.io/library/golang to v1.23.2

⚙️ Miscellaneous Tasks:
- *(container)* Update image docker.io/library/golang to 4f063a2

Version v1.0.0
--------------------
Released: 2024-09-29

💼 Other:
- Update go version to 1.22
- Add example kubernetes daemonset
- Add indentation to json body in response
- Add unit-test for update method
- Fix IPv4/IPv6 only update to dual stack domain

Version v0.1.0
--------------------
Released: 2024-09-28

🚀 Features:
- *(container)* Update image docker.io/library/golang to v1.22.0
- *(container)* Update image docker.io/library/golang to v1.23.0

🐛 Bug Fixes:
- *(deps)* Update module github.com/stretchr/testify to v1.9.0
- *(container)* Update image docker.io/library/golang to v1.22.1
- *(container)* Update image docker.io/library/golang to v1.22.2
- *(container)* Update image docker.io/library/golang to v1.22.3
- *(container)* Update image docker.io/library/golang to v1.22.4
- *(container)* Update image docker.io/library/golang to v1.22.5
- *(container)* Update image docker.io/library/golang to v1.23.1

⚙️ Miscellaneous Tasks:
- *(container)* Update image docker.io/library/golang to 7b575fe
- *(container)* Update image docker.io/library/golang to 7b297d9
- *(container)* Update image docker.io/library/golang to 0b55ab8
- *(container)* Update image docker.io/library/golang to 450e382
- *(container)* Update image docker.io/library/golang to d5302d4
- *(container)* Update image docker.io/library/golang to f43c6f0
- *(container)* Update image docker.io/library/golang to 2303a02
- *(container)* Update image docker.io/library/golang to a1dc408
- *(container)* Update image docker.io/library/golang to a0679ac
- *(container)* Update image docker.io/library/golang to 4950c1c
- *(container)* Update image docker.io/library/golang to c2010b9
- *(container)* Update image docker.io/library/golang to a66eda6
- *(container)* Update image docker.io/library/golang to c8736b8
- *(container)* Update image docker.io/library/golang to 829eff9
- *(container)* Update image docker.io/library/golang to 86a3c48

💼 Other:
- Add initial implementation of CI
- Do not use branch type for automerge
- Remove unneeded go get from script
- Run Renovate less frequently
- Adapt commands to be equal to other repos
- Add unit-tests as separate step
- Drop expose option
- Add binary artifact to outputs
- Add workflow for release

