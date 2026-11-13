
Bugs:

    BUG-3: midi stop — no All Notes Off (CC 123) sent on all 16 channels before halting clock.
    BUG-4: midi load <file>.mudl — LoadMudl not yet implemented (stub).
    BUG-5: midi monitor — returns immediately with port name instead of streaming live MIDI input events.
    BUG-6: midi record — returns immediately with no output or confirmation.
    BUG-7: audio host <name> — case-sensitive, requires exact title case Alsa. Should match case-insensitively.
    BUG-8: audio hosts — should list all available hosts (like midi ports), not the currently selected host.
    BUG-10: nu_plugin_midi / nu_plugin_live_audio — all plugin commands hang indefinitely, uninterruptible via Ctrl+C.

Enhancements:

    ENHANCE-1: midi port <name> — accept integer index or substring match in addition to full port name.
    ENHANCE-2: mush-shell — spacebar maps to stop during playback/record/monitor.
    ENHANCE-3: mush_midi_codec — symbolic CC names not supported, only numeric/hex.
    ENHANCE-4: All shells — suppress ALSA stderr noise during device enumeration, or provide opt-out via flag/env var.
    ENHANCE-5: audio host / audio device — display actual resolved default name alongside (default) indicator.
    ENHANCE-6: mush-shell — add top-level stop command to stop all active MIDI and audio.
    ENHANCE-7 → ENHANCE-8: nu_plugin_midi — help midi not found; add top-level midi command that prints usage.
