# Changelog

Currently, the changelog is built locally. It will be moved to CI once labels stabilize.

For now, a bit of preparation is required before you can run the script:
- fetch the srtool digests
- store them under the `digests` folder as `<chain>-srtool-digest.json`
- ensure the `.env` file is up to date with correct information

The content of the release notes is generated from the template files under the `scripts/ci/changelog/templates` folder.
For readability and maintenance, the template is split into several small snippets.

Run:
```
./bin/changelog <ref_since> [<ref_until>=HEAD]
```

For instance:
```
./bin/changelog parachains-v7.0.0-rc8
```

A file called `release-notes.md` will be generated and can be used for the release.

## ENV

You may use the following ENV for testing:

```
RUSTC_STABLE="rustc 1.56.1 (59eed8a2a 2021-11-01)"
RUSTC_NIGHTLY="rustc 1.57.0-nightly (51e514c0f 2021-09-12)"
PRE_RELEASE=true
HIDE_SRTOOL_ROCOCO=true
HIDE_SRTOOL_SHELL=true
REF1=statemine-v5.0.0
REF2=HEAD
DEBUG=1
NO_CACHE=1
```

By default, the template will include all the information, including the runtime data. For clients releases, we don't
need those and they can be skipped by setting the following env:
```
RELEASE_TYPE=client
```

## Considered labels

The following list will likely evolve over time and it will be hard to keep it in sync. In any case, if you want to find
all the labels that are used, search for `meta` in the templates. Currently, the considered labels are:

- Priority: C<N> labels
- Audit: D<N> labels
- E4 => new host function
- B0 => silent, not showing up
- B1-releasenotes (misc unless other labels)
- B5-client (client changes)
- B7-runtimenoteworthy (runtime changes)
- T6-XCM

Note that labels with the same letter are mutually exclusive. A PR should not have both `B0` and `B5`, or both `C1` and
`C9`. In case of conflicts, the template will decide which label will be considered.

## Dev and debugging

### Hot Reload

The following command allows **Hot Reload**:
```
fswatch templates -e ".*\.md$" | xargs -n1 -I{} ./bin/changelog statemine-v5.0.0
```
### Caching

By default, if the changelog data from Github is already present, the calls to the Github API will be skipped and the
local version of the data will be used. This is much faster. If you know that some labels have changed in Github, you
probably want to refresh the data. You can then either delete manually the `cumulus.json` file or `export NO_CACHE=1` to
force refreshing the data.
