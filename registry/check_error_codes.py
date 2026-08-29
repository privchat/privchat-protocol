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
#
# 注意:第一版只认 `const val CODE_*`,漏掉了裸字面量与非 CODE_ 前缀常量
# (BotMenuTransferHandler 的 `error(20901, ...)` 与 `BOT_BINDING_MISSING`),
# 造成"碰撞清零"的假阴性。现覆盖**任意名称**的常量。
SCANNERS = [
    ("rust-enum",  "privchat-protocol/src/error_code.rs",
     re.compile(r"^\s{4}(\w+)\s*=\s*(\d{4,5}),", re.M)),
    ("rust-const", "**/*.rs",
     re.compile(r"const\s+([A-Z][A-Z0-9_]*)\s*:\s*\w+\s*=\s*(\d{4,5})\s*;")),
    ("kotlin",     "**/*.kt",
     re.compile(r"const\s+val\s+([A-Z][A-Z0-9_]*)\s*:\s*Int\s*=\s*(\d{4,5})")),
    ("typescript", "**/*.ts",
     re.compile(r"const\s+([A-Z][A-Z0-9_]*)\s*(?::\s*number)?\s*=\s*(\d{5})")),
]

# 错误返回位置的**裸五位字面量** —— 业务码不得硬编码,必须走常量/registry。
BARE_LITERAL = [
    ("kotlin", "**/*.kt", re.compile(r"\.error\(\s*(\d{5})\s*,")),
    ("kotlin", "**/*.kt", re.compile(r"\.accepted\(\s*(\d{5})\s*,")),
    ("rust",   "**/*.rs", re.compile(r"::error\([^)]*?,\s*(\d{5})\s*,")),
    ("ts",     "**/*.ts", re.compile(r"case\s+(\d{5})\s*:")),
]

SKIP = ("/target/", "/node_modules/", "/build/", "/.git/", "/dist/",
        "/uniffi-kotlin-multiplatform-bindings/", "/_archive/", "/.gradle/",
        "/vendor/", "/godot-cpp/", "/examples/audio-cpp-app/",
        "/registry/fixtures/")   # 负向 fixture 仅供 self_test 单独扫描


def _registry_path(root: Path) -> Path:
    """支持从仓库根或 registry 目录本身调用(负向 fixture 自检用)。"""
    for cand in (root / "privchat-protocol/registry/error_codes.toml",
                 root / "error_codes.toml",
                 root.parent / "error_codes.toml"):
        if cand.exists():
            return cand
    raise SystemExit(f"registry not found from {root}")


def load_registry(root: Path):
    data = tomllib.loads(_registry_path(root).read_text())
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


def load_registry_raw(root: Path):
    """返回原始 list,用于检测 registry 自身的重复登记。"""
    data = tomllib.loads(_registry_path(root).read_text())
    return data["code"], data["segment"]


def scan_bare_literals(root: Path):
    """→ [(lang, code, path, line)] —— 错误返回位置的硬编码五位码。"""
    hits = []
    for lang, pattern, rx in BARE_LITERAL:
        candidates = list(root.glob(pattern))
        if pattern.startswith("**/"):
            candidates += list(root.glob(pattern[3:]))
        for path in candidates:
            p = str(path)
            if any(s in p for s in SKIP) or not path.is_file():
                continue
            try:
                text = path.read_text(errors="ignore")
            except OSError:
                continue
            for m in rx.finditer(text):
                line_no = text[: m.start()].count("\n") + 1
                line_txt = text.splitlines()[line_no - 1] if line_no <= len(text.splitlines()) else ""
                # 文档注释里的示例代码不是真实错误返回。
                if line_txt.lstrip().startswith(("*", "//", "///", "#")):
                    continue
                hits.append((lang, int(m.group(1)), str(path.relative_to(root)), line_no))
    return hits


def is_error_code(value: int, name: str) -> bool:
    """区分错误码与普通数值常量。

    放宽名称匹配后会扫到 BATCH_SIZE=1000 / KB=1024 / BPS_TOTAL=10000 这类
    与错误码无关的常量。判定依据:值必须落在错误码空间,且名称不能是
    明显的容量/尺寸/单位类。
    """
    if not (1 <= value <= 65535):
        return False
    # 按**词**匹配,不用子串:`DELIBERATE` 含 "rate"、`REPORT` 含 "port",
    # 子串匹配会把真错误码误判为普通常量(负向 fixture 曾因此漏检)。
    words = {w for w in name.lower().split("_") if w}
    NON_CODE = {"size", "capacity", "limit", "max", "min", "kb", "mb",
                "bytes", "bps", "timeout", "interval", "port", "entries",
                "distance", "dim", "total", "count", "batch", "ms",
                "rate", "delay", "duration", "width", "height",
                "threshold", "buffer", "window", "seed", "version",
                "default", "cap"}
    if words & NON_CODE:
        return False
    # 错误码空间:1-999 系统段、10000+ 业务段。1000-9999 未使用。
    return value < 1000 or value >= 10000


def scan(root: Path):
    """→ {code: [(lang, name, path), ...]}"""
    found = defaultdict(list)
    for lang, pattern, rx in SCANNERS:
        # `**/*.kt` 不匹配 root 同级文件,负向 fixture 因此被漏掉;
        # 显式补一次同级匹配。
        candidates = list(root.glob(pattern))
        if pattern.startswith("**/"):
            candidates += list(root.glob(pattern[3:]))
        for path in candidates:
            p = str(path)
            if any(s in p for s in SKIP) or not path.is_file():
                continue
            try:
                text = path.read_text(errors="ignore")
            except OSError:
                continue
            for m in rx.finditer(text):
                name, code = (m.group(1), m.group(2)) if m.lastindex == 2 else ("<literal>", m.group(1))
                if not is_error_code(int(code), name):
                    continue
                found[int(code)].append((lang, name, str(path.relative_to(root))))
    return found


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    codes, segments = load_registry(root)
    found = scan(root)

    problems = []

    # 0a. registry 自身:code 不得重复登记(dict 构造会静默覆盖,必须查原始 list)
    raw_codes, raw_segments = load_registry_raw(root)
    seen_reg = {}
    for c in raw_codes:
        if c["code"] in seen_reg:
            problems.append(
                f"[registry] {c['code']} 重复登记: {seen_reg[c['code']]} 与 {c['name']}")
        seen_reg[c["code"]] = c["name"]

    # 0b. registry 自身:segment 不得重叠
    ranges = []
    for sgm in raw_segments:
        lo, hi = (int(x) for x in sgm["range"].split("-"))
        for l2, h2, d2 in ranges:
            if lo <= h2 and l2 <= hi:
                problems.append(
                    f"[registry] 段位重叠: {sgm['domain']} {sgm['range']} 与 {d2} {l2}-{h2}")
        ranges.append((lo, hi, sgm["domain"]))

    # 0c. registry 自身:code 的 domain 必须落在对应 segment 内
    for c in raw_codes:
        dom = domain_of(c["code"], segments)
        if dom is None:
            problems.append(f"[registry] {c['code']} ({c['name']}) 不在任何 segment 内")
        elif dom != c["domain"]:
            problems.append(
                f"[registry] {c['code']} ({c['name']}) 标注 domain={c['domain']},"
                f"但所在段位属于 {dom}")

    # 0d. 错误返回位置的硬编码五位码。
    #     TS 的 `case 21500:` 是分支消费而非**定义**语义,不会造成碰撞;
    #     列为 warning 供逐步整改。Kotlin/Rust 的 `.error(21501, ...)` 是
    #     在产生错误码,属硬失败。
    warnings = []
    for lang, code, path, line in scan_bare_literals(root):
        msg = f"{path}:{line} 直接使用字面量 {code}({lang})"
        if lang == "ts":
            warnings.append(f"[warn] {msg} —— 建议改用常量")
        else:
            problems.append(f"[硬编码] {msg}——应改用常量/registry")

    # 1. 同一个码被赋予**语义不同**的多个名称 = 碰撞。
    #    各语言镜像同一语义(CODE_INVALID_PARAMS 对 InvalidParams)是正常做法,
    #    只有归一化后仍不同才算真碰撞。
    def norm(name: str) -> str:
        """归一化名称以比较语义。

        各语言的命名前缀不同(Rust `UploadSessionBusy` / TS
        `CODE_SESSION_BUSY`),只要去掉域前缀后一致就视为同一语义 ——
        否则纯命名风格差异会淹没真正的碰撞信号。
        """
        return name.removeprefix("CODE_").replace("_", "").lower()

    DOMAIN_PREFIXES = ("upload", "sync", "bot", "botlifecycle", "transfer",
                       "channel", "serverevent", "game", "message",
                       "system", "device", "file", "qrcode", "user")

    def same_semantic(a: str, b: str) -> bool:
        """域前缀可有可无:`SESSION_BUSY` 与 `UploadSessionBusy` 是同一语义。"""
        x, y = norm(a), norm(b)
        if x == y:
            return True
        for p_ in DOMAIN_PREFIXES:
            if x.removeprefix(p_) == y.removeprefix(p_):
                return True
        return False

    for code, uses in sorted(found.items()):
        raw = [n for _, n, _ in uses if n != "<literal>"]
        names = set()
        for n in raw:
            if not any(same_semantic(n, m) for m in names):
                names.add(n)
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
            if not same_semantic(name, want):
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
    if warnings:
        print(f"\n{len(warnings)} 项待整改(不阻断):")
        for w in warnings[:5]:
            print("  " + w)
        if len(warnings) > 5:
            print(f"  ... 另有 {len(warnings) - 5} 项")
    return 0


if __name__ == "__main__":
    sys.exit(main())
