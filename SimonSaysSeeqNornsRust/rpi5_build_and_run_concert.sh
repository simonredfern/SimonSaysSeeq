#!/bin/bash
#
# SimonSaysSeeq concert wrapper.
#
# Thin wrapper around rpi5_build_and_run.sh that runs the sequencer with the
# binary's --concert flag, which pins the log filter to WARN (RUST_LOG
# ignored) so a misconfigured launcher can't accidentally turn on info-level
# per-step / per-button logging during a show. WARN keeps useful diagnostic
# signal (failed MIDI sends, queue overruns) while filtering the noisy info
# chatter that runs on the audio path.
#
# Behaviour vs. the base script:
#   - Same build, same dependencies, same binary — there is no separate
#     "concert build". The only difference is the runtime flag.
#   - For `run`, --concert is appended to the args passed to the binary.
#   - For every other command (build, setup, service, info, ...), this is
#     a transparent pass-through to rpi5_build_and_run.sh.
#
# Usage:
#   ./rpi5_build_and_run_concert.sh                  # defaults to: run
#   ./rpi5_build_and_run_concert.sh build            # build (release) only
#   ./rpi5_build_and_run_concert.sh run              # build + run in concert mode
#   ./rpi5_build_and_run_concert.sh run -- --foo     # extra binary args after --concert
#   ./rpi5_build_and_run_concert.sh service install  # install systemd service
#                                                    # in concert mode (autostart
#                                                    # at boot will pass --concert)
#
# Mechanism: when forwarding a `service` command, this wrapper exports
# SSSEQ_INSTALL_CONCERT_MODE=1, which the base script reads at install time
# to bake --concert into the generated /usr/local/bin/simonsaysseeq_wrapper.sh.

set -e

cd "$(dirname "$0")"

BASE_SCRIPT="./rpi5_build_and_run.sh"

if [ ! -x "$BASE_SCRIPT" ]; then
    echo "ERROR: $BASE_SCRIPT not found or not executable next to this wrapper." >&2
    exit 1
fi

# Default to "run" if invoked with no arguments at all.
COMMAND="${1:-run}"
shift || true

case "$COMMAND" in
    run)
        # Inject --concert as the first binary arg. Any extra args the user
        # passed (e.g. another `--something` after `--`) are forwarded after.
        # The base script forwards everything after `--` to the binary.
        exec "$BASE_SCRIPT" run -- --concert "$@"
        ;;
    service)
        # Tell the base script to bake --concert into the systemd wrapper it
        # regenerates during `service install`. Harmless for other service
        # actions (start/stop/status/enable/disable) — they don't regenerate
        # the wrapper, so the env var is just ignored.
        export SSSEQ_INSTALL_CONCERT_MODE=1
        exec "$BASE_SCRIPT" service "$@"
        ;;
    *)
        # build, setup, info, clean, help, ... — pass through unchanged.
        exec "$BASE_SCRIPT" "$COMMAND" "$@"
        ;;
esac
