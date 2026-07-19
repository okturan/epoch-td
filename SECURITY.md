# Security policy

## Supported version

Security fixes are made on the `main` branch and flow to the version published
on GitHub Pages. Historical commits, downloaded copies, and modified mirrors are
not supported separately.

## Reporting a vulnerability

Please use [GitHub's private vulnerability reporting form](https://github.com/okturan/epoch-td/security/advisories/new)
instead of opening a public issue. Include the affected file or hosted URL, the
browser and operating system you tested, clear reproduction steps, and the
impact you expect.

This project is a dependency-light, client-side game. Useful reports include:

- a way for repository-controlled game data or markup to execute unexpected
  script in the hosted page;
- an unsafe dependency, build, or GitHub Pages deployment path;
- a reproducible browser-storage issue that exposes data outside this origin;
- a security-relevant difference between the committed game and the published
  site.

Game balance exploits, score manipulation in a user's own browser, unsupported
mobile layouts, and bugs that only affect a locally modified copy are ordinary
bug reports rather than vulnerabilities.

Do not include credentials, personal data, or destructive proof-of-concept
payloads. A minimal synthetic example is enough. The maintainer will coordinate
validation, remediation, and disclosure through the private advisory.
