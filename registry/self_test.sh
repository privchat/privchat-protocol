#!/usr/bin/env bash
# 检查器自检:先证明它能抓到人为碰撞,再跑真实扫描。
#
# 负向 fixture 复制到临时目录再扫描 —— 真实扫描已把 registry/fixtures
# 排除在外(它不是生产代码),不能直接对源目录跑。
set -u
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
HERE="$(cd "$(dirname "$0")" && pwd)"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
cp "$HERE/fixtures/negative_collision.kt" "$TMP/"
cp "$HERE/error_codes.toml" "$TMP/"

echo "== 负向 fixture:期望失败 =="
if python3 "$HERE/check_error_codes.py" "$TMP" >/dev/null 2>&1; then
    echo "FAIL: 检查器未能发现人为碰撞——它已失效"
    exit 1
fi
echo "OK: 检查器正确报出人为碰撞"

echo "== 真实扫描:期望通过 =="
python3 "$HERE/check_error_codes.py" "$ROOT"
