#!/usr/bin/env bash

#
# Trigger a release
#

set -e -u -o pipefail

# We have to run this locally because tags created from workflows do not
# trigger new workflows.
# "This prevents you from accidentally creating recursive workflow runs."

echo "CREATING A RELEASE WITH:"

METADATA="$(cargo metadata --format-version=1 --no-deps)"
VERSION="$(echo $METADATA | jq -r '.packages[0].version')"
echo "VERSION $VERSION"
TAGNAME="v$VERSION"
echo "TAGNAME $TAGNAME"

echo ""
echo "STEPS:"
echo ""
echo "- UPDATE 'CHANGELOG.md'"
echo ""
echo "- ENSURE YOU ARE ON THE MAIN BRANCH"
echo ""
echo "- RUN 'cargo publish --allow-dirty --dry-run'"
echo ""
echo "- PUSH A NEW COMMIT WITH MESSAGE 'Release $VERSION'"
echo ""
echo "- RUN 'cargo publish'"
echo ""
echo "- CREATE A NEW TAG, SEE BELOW"
echo ""

NOTES="See [CHANGELOG.md](https://github.com/rikhuijzer/transformrs/blob/main/CHANGELOG.md) for more information about changes since the last release."

echo "Ready to create a new tag, which WILL TRIGGER A RELEASE with the following release notes:"
echo "\"$NOTES\""
echo ""
read -p "Are you sure? Type YES to continue. " REPLY

if [[ $REPLY == "YES" ]]; then
    echo ""
    git tag -a $TAGNAME -m "$NOTES"
    git push origin $TAGNAME
    exit 0
else
    echo ""
    echo "Did not receive YES, aborting"
    exit 1
fi

I read about ingesting PDFs in Google Gemini here on HN last week [1]. This and some other thoughts that I had on AI [2] made me want to create a summarize PDF command line utility. Just point the utility to a PDF and get a summary back. However, I personally prefer to write CLI tools in Rust since those binaries are fast, small, and easy to distribute via `cargo install` (or `cargo binstall`). So that was a problem. I wanted to use a cloud provider since my system doesn't have enough RAM for most state-of-the-art models, but at the same time, I didn't want to require users of the tool to use the same API provider as me.

That's why I created the `transformrs` Rust library [3]. The name is from "transform", which is essentially what AI models do, and "Rust". It is also a nudge to the transformers algorithm.

[1]: https://news.ycombinator.com/item?id=42952605

[2]: https://huijzer.xyz/posts/ai-learning-rate/

[3]: https://github.com/transformrs/transformrs