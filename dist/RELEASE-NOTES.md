# aibox v0.17.10

Bug-fix release: `[aibox].version = "latest"` no longer fails validation.
No processkit version change — still compatible with v0.8.0.

## Fix: `[aibox].version = "latest"` rejected by config validator

`aibox sync` was failing with:

```
ERR Invalid version 'latest': must be valid semver: unexpected character 'l' while parsing major version number
```

The `validate()` method in `config.rs` was calling `semver::Version::parse` on
`[aibox].version` unconditionally. The `[processkit].version` section already
had the `"latest"` exemption guard from v0.17.9; the `[aibox]` section did not.

Fixed by adding the same guard. A regression test was added to prevent
recurrence.
