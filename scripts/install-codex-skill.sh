#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
source_dir="$repo_root/skills/codex/polaris"

if [ ! -f "$source_dir/SKILL.md" ]; then
  echo "Bundled Polaris skill not found at $source_dir" >&2
  exit 1
fi

if [ -n "${POLARIS_CODEX_SKILLS_DIR:-}" ]; then
  skills_dir=$POLARIS_CODEX_SKILLS_DIR
else
  if [ -n "${CODEX_HOME:-}" ]; then
    codex_home=$CODEX_HOME
  elif [ -n "${HOME:-}" ] && [ -d "$HOME/.codex-app" ]; then
    codex_home="$HOME/.codex-app"
  elif [ -n "${HOME:-}" ]; then
    codex_home="$HOME/.codex"
  else
    echo "Set CODEX_HOME, HOME, or POLARIS_CODEX_SKILLS_DIR before installing the skill" >&2
    exit 1
  fi
  skills_dir="$codex_home/skills"
fi

target_dir="$skills_dir/polaris"
mkdir -p "$skills_dir"
temp_dir=$(mktemp -d "$skills_dir/.polaris-install.XXXXXX")

cleanup() {
  if [ -n "${temp_dir:-}" ] && [ -d "$temp_dir" ]; then
    rm -rf "$temp_dir"
  fi
}
trap cleanup EXIT INT TERM

cp -R "$source_dir/." "$temp_dir/"
rm -rf "$target_dir"
mv "$temp_dir" "$target_dir"
temp_dir=

echo "Installed Polaris Codex skill to $target_dir"
