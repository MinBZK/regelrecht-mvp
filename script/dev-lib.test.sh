#!/usr/bin/env bash
# Dekt de mold-eis in dev_preflight (script/dev-lib.sh): alleen op x86_64 Linux
# linkt cargo met mold, dus alleen daar mag het ontbreken ervan `just dev`
# tegenhouden. `uname` en de tools staan als stub op een afgeschermd PATH, zodat
# de uitkomst niet afhangt van de machine waarop de test draait.
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
lib="$here/dev-lib.sh"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
bash_bin="$(command -v bash)"

pass=0
fail=0

# Bouwt een PATH met alleen de gevraagde stubs. $1 = uname -s, $2 = uname -m,
# $3.. = tools die aanwezig zijn.
make_path() {
    local dir="$tmp/$RANDOM$RANDOM" os="$1" arch="$2"
    shift 2
    mkdir -p "$dir"
    cat >"$dir/uname" <<STUB
#!$bash_bin
case "\${1:-}" in -s) echo "$os" ;; -m) echo "$arch" ;; esac
STUB
    local tool
    for tool in "$@"; do printf '#!%s\nexit 0\n' "$bash_bin" >"$dir/$tool"; done
    chmod +x "$dir"/*
    echo "$dir"
}

# $1 = naam, $2 = verwachte exitcode, $3 = uname -s, $4 = uname -m,
# $5 = patroon dat wel (+) of niet (-) in de uitvoer moet staan, $6.. = tools.
check() {
    local name="$1" want="$2" os="$3" arch="$4" needle="$5"
    shift 5
    local path out status
    path="$(make_path "$os" "$arch" "$@")"
    out="$(PATH="$path" "$bash_bin" -c "source '$lib'; dev_preflight --rust" 2>&1)"
    status=$?
    local ok=true
    [ "$status" -eq "$want" ] || ok=false
    case "$needle" in
        +*) [[ "$out" == *"${needle#+}"* ]] || ok=false ;;
        -*) [[ "$out" != *"${needle#-}"* ]] || ok=false ;;
    esac
    if [ "$ok" = true ]; then
        pass=$((pass + 1))
    else
        fail=$((fail + 1))
        printf 'FAIL %s: exit %s (verwacht %s)\n%s\n' "$name" "$status" "$want" "$out"
    fi
}

check "x86_64 Linux zonder mold weigert" 1 Linux x86_64 "+mold" docker cargo
check "x86_64 Linux met mold start" 0 Linux x86_64 "-Missing" docker cargo mold
check "macOS arm64 zonder mold start" 0 Darwin arm64 "-mold" docker cargo
check "macOS x86_64 zonder mold start" 0 Darwin x86_64 "-mold" docker cargo
check "aarch64 Linux zonder mold start" 0 Linux aarch64 "-mold" docker cargo
check "macOS zonder cargo weigert nog steeds" 1 Darwin arm64 "+cargo" docker

# dev_start en dev_stop: wat `just dev-down` stopt moet de hele procesboom zijn,
# niet alleen de shell eromheen. De dienst start hier zelf een kind (zoals npm
# node start en cargo de binary); overleeft dat kind dev_stop, dan houdt het in
# het echt zijn poort vast en faalt de volgende `just dev`. $1 = naam, $2 = PATH
# waarmee de bibliotheek draait (met of zonder setsid).
check_stop() {
    local name="$1" path="$2" dir="$tmp/$RANDOM$RANDOM"
    mkdir -p "$dir"
    (
        cd "$dir" || exit 1
        PATH="$path" PIDFILE=pids COMPOSE=true "$bash_bin" -c "
            source '$lib'
            dev_start dienst dienst.log 'sleep 300 & echo \$! > kind.pid; wait' > /dev/null
            for _ in 1 2 3 4 5 6 7 8 9 10; do [ -s kind.pid ] && break; sleep 0.2; done
            cp kind.pid kind.pid.bewaard
            dev_stop > /dev/null
        "
    )
    local kind
    kind="$(cat "$dir/kind.pid.bewaard" 2>/dev/null)"
    if [ -n "$kind" ] && ! kill -0 "$kind" 2>/dev/null; then
        pass=$((pass + 1))
    else
        fail=$((fail + 1))
        printf 'FAIL %s: kindproces %s draait nog na dev_stop\n' "$name" "${kind:-<niet gestart>}"
        [ -n "$kind" ] && kill "$kind" 2>/dev/null
    fi
}

check_stop "dev_stop stopt de hele procesboom (setsid)" "$PATH"

# Zonder setsid(1), zoals op macOS: een PATH met alleen wat de bibliotheek dan
# nog nodig heeft, zodat de perl-tak draait.
if command -v perl >/dev/null 2>&1; then
    zonder_setsid="$tmp/zonder-setsid"
    mkdir -p "$zonder_setsid"
    for tool in bash perl sleep rm cp; do ln -s "$(command -v "$tool")" "$zonder_setsid/$tool"; done
    check_stop "dev_stop stopt de hele procesboom (perl)" "$zonder_setsid"
fi

printf '%d geslaagd, %d gefaald\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
