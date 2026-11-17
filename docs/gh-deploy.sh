#!/bin/bash
# Build the book and publish it on the gh-pages branch, like `mkdocs gh-deploy`.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
# The deploy commit names HEAD: refuse to publish uncommitted changes under its sha.
git diff --quiet HEAD || { echo "Uncommitted changes, commit them first" >&2; exit 1; }
make -C docs html

REV=$(git rev-parse --short HEAD)
WORKTREE=$(mktemp -d)

# Like `mkdocs gh-deploy`, the local gh-pages branch is owned by this script:
# it is reset to the remote one, or dropped if a first push failed.
if git ls-remote --exit-code --heads origin gh-pages > /dev/null
then
    git fetch origin gh-pages
    git worktree add -B gh-pages "${WORKTREE}" FETCH_HEAD
else
    if git show-ref --quiet --verify refs/heads/gh-pages
    then
        git branch -D gh-pages
    fi
    git worktree add --orphan -b gh-pages "${WORKTREE}"
fi
trap 'git worktree remove --force "${WORKTREE}"' EXIT

git -C "${WORKTREE}" rm -r -q --ignore-unmatch .
cp -r docs/build/html/. "${WORKTREE}/"
git -C "${WORKTREE}" add -A
if git -C "${WORKTREE}" diff --cached --quiet
then
    echo "gh-pages already matches ${REV}, nothing to publish"
    exit 0
fi
git -C "${WORKTREE}" commit -q -m "docs: deploy ${REV}"
git -C "${WORKTREE}" push origin gh-pages
