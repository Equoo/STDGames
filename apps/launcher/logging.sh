
# LEVEL: 0=fatal only, 1=+error, 2=+warn, 3=+info(default), 4=+debug, 5=+dim
LOG_LEVEL=${LEVEL:-3}

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[0;37m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m' # No Color

# Basic
print()    { echo -e "${NC}$*${NC}"; }
success()  { echo -e "${GREEN}[OK]${NC} $*"; }
bold()     { echo -e "${BOLD}$*${NC}"; }
dim()      { echo -e "${DIM}$*${NC}"; }

_log() {
    local level=$1 color=$2 label=$3; shift 3
    [ "$level" -le "$LOG_LEVEL" ] && echo -e "${color}${label}${NC} $*"
}
_err() {
    local level=$1 color=$2 label=$3; shift 3
    [ "$level" -le "$LOG_LEVEL" ] && echo -e "${color}${label}${NC} $*" >&2
}

debug()   { _log 4 "${DIM}"     "[DEBUG]" "$*"; }
info()    { _log 3 "${CYAN}"    "[INFO] " "$*"; }
warn()    { _log 2 "${YELLOW}"  "[WARN] " "$*"; }
error()   { _err 1 "${RED}"     "[ERROR]" "$*"; }
fatal()   { _err 0 "${RED}${BOLD}" "[FATAL]" "$*"; exit 1; }

# Section header
header() {
    local text="$*"
    local len=${#text}
    local line=$(printf '─%.0s' $(seq 1 $((len + 4))))
    echo -e "${BLUE}${BOLD}┌${line}┐${NC}"
    echo -e "${BLUE}${BOLD}│  ${text}  │${NC}"
    echo -e "${BLUE}${BOLD}└${line}┘${NC}"
}

# Step with index
step() {
    local n=$1; shift
    echo -e "${MAGENTA}[${n}]${NC} $*"
}
