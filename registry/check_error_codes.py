#!/usr/bin/env python3
"""跨仓错误码门禁。

历次碰撞(20900/20902 与 20920-20923)都不是 Rust enum 内部重复 —— 它们是
**跨语言、跨仓库的手写常量**互相撞车,单仓单语言的测试一个都发现不了。
本脚本扫描所有语言的错误码定义点,与 registry/error_codes.toml 交叉校验。

用法:
    python3 registry/check_error_codes.py <repo-root>
退出码 0 = 通过;1 = 发现碰撞或未登记码。
"""
import re
import sys
import tomllib
from pathlib import Path
from collections import defaultdict

# (语言, glob, 正则) —— 正则须捕获 (name, code)
SCANNERS = [
    ("rust-enum",  "privchat-protocol/src/error_code.rs",
     re.compile(r"^\s{4}(\w+)\s*=\s*(\d{4,5}),", re.M)),
    ("rust-const", "**/*.rs",
     re.compile(r"const\s+(CODE_\w+)\s*:\s*\w+\s*=\s*(\d{4,5})\s*;")),
    ("kotlin",     "**/*.kt",
     re.compile(r"const\s+val\s+(CODE_\w+)\s*:\s*Int\s*=\s*(\d{4,5})")),
    ("typescript", "**/*.ts",
     re.compile(r"(?:case|=)\s*(\d{5})\s*[:;]")),
]

SKIP = ("/target/", "/node_modules/", "/build/", "/.git/", "/dist/")


def load_registry(root: Path):
    data = tomllib.loads((root / "privchat-protocol/registry/error_codes.toml").read_text())
    codes = {c["code"]: c for c in data["code"]}
    segments = []
    for s in data["segment"]:
        lo, hi = s["range"].split("-")
        segments.append((int(lo), int(hi), s["domain"]))
    return codes, segments


def domain_of(code: int, segments):
    for lo, hi, dom in segments:
        if lo <= code <= hi:
            return dom
    return None


def scan(root: Path):
    """→ {code: [(lang, name, path), ...]}"""
    found = defaultdict(list)
    for lang, pattern, rx in SCANNERS:
        for path in root.glob(pattern):
            p = str(path)
            if any(s in p for s in SKIP) or not path.is_file():
                continue
            try:
                text = path.read_text(errors="ignore")
            except OSError:
                continue
            for m in rx.finditer(text):
                name, code = (m.group(1), m.group(2)) if m.lastindex == 2 else ("<literal>", m.group(1))
                found[int(code)].append((lang, name, str(path.relative_to(root))))
    return found


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    codes, segments = load_registry(root)
    found = scan(root)

    problems = []

    # 1. 同一个码被赋予**语义不同**的多个名称 = 碰撞。
    #    各语言镜像同一语义(CODE_INVALID_PARAMS 对 InvalidParams)是正常做法,
    #    只有归一化后仍不同才算真碰撞。
    def norm(name: str) -> str:
        return name.removeprefix("CODE_").replace("_", "").lower()

    for code, uses in sorted(found.items()):
        names = {norm(n) for _, n, _ in uses if n != "<literal>"}
        if len(names) > 1:
            problems.append(
                f"[碰撞] {code} 有 {len(names)} 套语义: " +
                ", ".join(f"{n} ({lang} {p})" for lang, n, p in uses if n != "<literal>")
            )

    # 2. 已登记码的名称必须与 registry 一致
    for code, uses in sorted(found.items()):
        if code not in codes:
            continue
        want = codes[code]["name"]
        for lang, name, p in uses:
            if name == "<literal>":
                continue
            if norm(name) != want.lower():
                problems.append(f"[不符] {code} registry 为 {want},但 {p} 定义为 {name}")

    # 3. 定义了却未登记
    for code, uses in sorted(found.items()):
        if code in codes or code < 20000:
            continue
        real = [u for u in uses if u[1] != "<literal>"]
        if real:
            problems.append(
                f"[未登记] {code} 在代码中定义但不在 registry: " +
                ", ".join(f"{n} ({p})" for _, n, p in real)
            )

    if problems:
        print(f"错误码门禁未通过({len(problems)} 项):\n")
        for p in problems:
            print("  " + p)
        return 1

    print(f"错误码门禁通过:扫描 {len(found)} 个码,registry 登记 {len(codes)} 个")
    return 0


if __name__ == "__main__":
    sys.exit(main())
