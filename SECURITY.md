# Security

## Reporting

Report vulnerabilities privately through GitHub's
[security advisories](https://github.com/UnhingedSoftware/tapline/security/advisories/new).
Please do not open a public issue for anything exploitable.

## What is worth reporting

tapline installs files whose names come from Workshop manifests, and anyone can
publish a Workshop item. It also handles Steam credentials. So in rough order of
how much a report matters:

- **A path escaping the install root.** Manifest filenames are attacker
  controlled. `..`, absolute paths, or a symlink that redirects a later write
  should all fail the install, not be skipped.
- **Content accepted without verification.** Every chunk's SHA-1 is checked
  against its content-addressed id before it reaches the disk. Anything that
  writes bytes that were not checked is a hole, and it is what makes fetching
  over a plain-HTTP lancache safe.
- **A credential leaving where it should not.** Passwords are encrypted with
  Steam's RSA key and never written to disk; only the refresh token persists.
  A password reaching a log, an argument list, an error message or the token
  file is a bug.
- **Anything executed from downloaded content.** Depots ship
  `installscript.vdf`. tapline parses and reports it and must never run it —
  installing a game server is not a remote-code-execution primitive.
- **Decompression that is not bounded** by the size the manifest declares.

## What is not a vulnerability

- Downloading content the signed-in account is entitled to. That is the tool's
  purpose, and it is what the real client does.
- Anonymous logon reaching anonymously accessible depots.
- `--password` putting a password in your shell history and process list. That
  is documented in `--help`; `--password-stdin` exists for anything scripted.

## Supported versions

The latest release. This is pre-1.0 and there are no backports.
