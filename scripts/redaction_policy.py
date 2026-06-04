"""Generic public redaction checks for ANIMEM JSON event payloads."""

from __future__ import annotations

import json
import os
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable


def _join(*parts: str) -> str:
    return "".join(parts)


@dataclass(frozen=True)
class Leak:
    pointer: str
    leak_class: str


def _normalize_key(value: str) -> str:
    return re.sub(r"[^a-z0-9]", "", value.lower())


FORBIDDEN_KEYS = tuple(
    _normalize_key(key)
    for key in (
        "token",
        "access_token",
        "refresh_token",
        "id_token",
        "api_key",
        "apikey",
        "secret",
        "password",
        "passwd",
        "passphrase",
        "authorization",
        "cookie",
        "credential",
        "credentials",
        "private_key",
        "ssh_key",
        "session_id",
        "prompt",
        "system_prompt",
        "messages",
        "transcript",
        "cwd",
        "path",
        "source_path",
        "local_path",
        "absolute_path",
        "archive_path",
        "document_text",
        "raw_text",
        "source_text",
        "request",
        "response",
        "env",
        "user_name",
        "person_name",
        "organization_name",
        "provider",
        "provider_endpoint",
        "provider_url",
        "model",
        "model_name",
    )
)

PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    (
        "credential",
        re.compile(_join(r"s", r"k-[A-Za-z0-9_-]{8,}", r"|Bearer", r"\s+[A-Za-z0-9._:-]{16,}")),
    ),
    (
        "private-ip",
        re.compile(
            _join(
                r"(?<!\d)(?:",
                r"10\.\d+\.\d+\.\d+",
                r"|172\.(?:1[6-9]|2[0-9]|3[0-1])\.\d+\.\d+",
                r"|192\.168\.\d+\.\d+",
                r")(?!\d)",
            )
        ),
    ),
    (
        "local-path",
        re.compile(_join(r"/", r"Users/", r"|/", r"home/", r"|/", r"vol\d+", r"/")),
    ),
    ("endpoint-url", re.compile(_join(r"https?", r"://"), re.IGNORECASE)),
    ("private-key", re.compile(_join(r"-----BEGIN ", r"[A-Z ]*", r"PRIVATE KEY-----"))),
)


class PayloadJsonError(Exception):
    pass


def load_json(path: str | Path) -> Any:
    try:
        with open(path, "r", encoding="utf-8") as fh:
            return json.load(fh)
    except json.JSONDecodeError as exc:
        raise PayloadJsonError("invalid JSON payload") from exc


def load_denylist_from_env() -> list[str]:
    path = os.environ.get("ANIMEM_PUBLIC_DENYLIST")
    if not path:
        return []
    denylist_path = Path(path)
    if not denylist_path.is_file():
        raise FileNotFoundError("ANIMEM_PUBLIC_DENYLIST is not a file")

    terms: list[str] = []
    with denylist_path.open("r", encoding="utf-8") as fh:
        for raw in fh:
            term = raw.strip()
            if not term or term.startswith("#"):
                continue
            terms.append(term.lower())
    return terms


def find_leaks(value: Any, denylist: Iterable[str] | None = None) -> list[Leak]:
    leaks: list[Leak] = []
    normalized_denylist = tuple(term.lower() for term in (denylist or ()))
    _walk(value, "", leaks, normalized_denylist)
    return leaks


def _escape_pointer_segment(segment: str) -> str:
    return segment.replace("~", "~0").replace("/", "~1")


def _key_leak_class(key: str, denylist: tuple[str, ...]) -> str | None:
    normalized = _normalize_key(key)
    if any(forbidden in normalized for forbidden in FORBIDDEN_KEYS):
        return "forbidden-key"
    lowered = key.lower()
    if any(term in lowered for term in denylist):
        return "denylist"
    for leak_class, pattern in PATTERNS:
        if pattern.search(key):
            return leak_class
    return None


def _value_leaks(value: str, pointer: str, leaks: list[Leak], denylist: tuple[str, ...]) -> None:
    lowered = value.lower()
    if any(term in lowered for term in denylist):
        leaks.append(Leak(pointer or "/", "denylist"))
    for leak_class, pattern in PATTERNS:
        if pattern.search(value):
            leaks.append(Leak(pointer or "/", leak_class))


def _walk(value: Any, pointer: str, leaks: list[Leak], denylist: tuple[str, ...]) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            key_text = str(key)
            key_class = _key_leak_class(key_text, denylist)
            segment = "<redacted-key>" if key_class else _escape_pointer_segment(key_text)
            child_pointer = f"{pointer}/{segment}"
            if key_class:
                leaks.append(Leak(child_pointer, key_class))
            _walk(child, child_pointer, leaks, denylist)
    elif isinstance(value, list):
        for idx, child in enumerate(value):
            _walk(child, f"{pointer}/{idx}", leaks, denylist)
    elif isinstance(value, bool) or value is None:
        return
    elif isinstance(value, (str, int, float)):
        _value_leaks(str(value), pointer, leaks, denylist)


def format_leaks(leaks: Iterable[Leak]) -> list[str]:
    return [f"payload leak {leak.leak_class} at {leak.pointer}" for leak in leaks]
