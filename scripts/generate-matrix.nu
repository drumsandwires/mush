# scripts/generate-matrix.nu
# Parses nextest JSON output and splices the updated coverage matrix
# into the COVERAGE-MATRIX sentinel block in TEST.md.

def cell [results: list, shell: string, command: string] {
    let pattern = $"test_($shell)_($command)"
    let matches = ($results | where name =~ $pattern)
    if ($matches | is-empty) { return "⬜" }
    let failed = ($matches | where status == "fail")
    if ($failed | is-empty) { "✅" } else { "❌" }
}

def main [] {
    let results = (
        open target/nextest-results.json
        | lines
        | each { |line| $line | from json }
        | where type == "test"
        | select name status
    )

    let matrix = [
        "| Command                     | Tier  | mish                              | mash                              | mush                              | nu_midi                               | nu_audio                              | Blocker   |",
        "|------------------------------|-------|-----------------------------------|-----------------------------------|-----------------------------------|---------------------------------------|---------------------------------------|-----------|",
        $"| `version`                   | I+P   | (cell $results mi version)        | (cell $results ma version)        | (cell $results mu version)        | (cell $results n_mi version)          | (cell $results n_la version)          |           |",
        $"| `ports`                     | I+P   | (cell $results mi ports)          | N/A                               | (cell $results mu ports)          | (cell $results n_mi ports)            | N/A                                   |           |",
        $"| `port` (get)                | I+P   | (cell $results mi port_get)       | N/A                               | (cell $results mu port_get)       | (cell $results n_mi port_get)         | N/A                                   |           |",
        $"| `port <name>` (set)         | I+P   | (cell $results mi port_set)       | N/A                               | (cell $results mu port_set)       | (cell $results n_mi port_set)         | N/A                                   | #79 #80   |",
        $"| `load song.mid`             | I+P   | (cell $results mi load_mid)       | N/A                               | (cell $results mu load_mid)       | (cell $results n_mi load_mid)         | N/A                                   | #90       |",
        $"| `load song.mudl`            | I+P   | (cell $results mi load_mudl)      | N/A                               | (cell $results mu load_mudl)      | (cell $results n_mi load_mudl)        | N/A                                   | #90       |",
        $"| `play`                      | I+S   | (cell $results mi play)           | N/A                               | (cell $results mu play)           | (cell $results n_mi play)             | N/A                                   | #82 #83   |",
        $"| `stop`                      | I+P   | (cell $results mi stop)           | N/A                               | (cell $results mu stop)           | (cell $results n_mi stop)             | N/A                                   |           |",
        $"| `pipeline play`             | P     | N/A                               | N/A                               | N/A                               | (cell $results n_mi pipeline_play)    | N/A                                   |           |",
        $"| `pipeline play filtered`    | P     | N/A                               | N/A                               | N/A                               | (cell $results n_mi pipeline_filtered)| N/A                                   |           |",
        $"| `send <msg>`                | U+I+P | (cell $results mi send)           | N/A                               | (cell $results mu send)           | (cell $results n_mi send)             | N/A                                   |           |",
        "| `monitor`                   | S     | 🚧                                | N/A                               | 🚧                                | 🚧                                    | N/A                                   | #86 #87   |",
        "| `record`                    | S     | 💨                                | N/A                               | 💨                                | 💨                                    | N/A                                   |           |",
        $"| `save song.mudl`            | I+P   | (cell $results mi save_mudl)      | N/A                               | (cell $results mu save_mudl)      | (cell $results n_mi save_mudl)        | N/A                                   |           |",
        $"| `save song.mid`             | I+P   | (cell $results mi save_mid)       | N/A                               | (cell $results mu save_mid)       | (cell $results n_mi save_mid)         | N/A                                   |           |",
        $"| `audio hosts`               | I+P   | N/A                               | (cell $results ma hosts)          | (cell $results mu hosts)          | N/A                                   | (cell $results n_la hosts)            |           |",
        $"| `audio host` (get)          | I+P   | N/A                               | (cell $results ma host_get)       | (cell $results mu host_get)       | N/A                                   | (cell $results n_la host_get)         |           |",
        $"| `audio host <name>` (set)   | I+P   | N/A                               | (cell $results ma host_set)       | (cell $results mu host_set)       | N/A                                   | (cell $results n_la host_set)         |           |",
        $"| `audio devices`             | I+P   | N/A                               | (cell $results ma devices)        | (cell $results mu devices)        | N/A                                   | (cell $results n_la devices)          |           |",
        $"| `audio device` (get)        | I+P   | N/A                               | (cell $results ma device_get)     | (cell $results mu device_get)     | N/A                                   | (cell $results n_la device_get)       |           |",
        "| `audio device <name>` (set) | I+P   | N/A                               | 🔮                                | 🔮                                | N/A                                   | 🔮                                    |           |",
    ]

    let table = ($matrix | str join "\n")

    let doc = (open TEST.md)
    let before = ($doc | split row "<!-- COVERAGE-MATRIX:BEGIN -->" | first)
    let after  = ($doc | split row "<!-- COVERAGE-MATRIX:END -->"   | last)

    let updated = $"($before)<!-- COVERAGE-MATRIX:BEGIN -->\n($table)\n<!-- COVERAGE-MATRIX:END -->($after)"
    $updated | save -f TEST.md

    print "Coverage matrix updated in TEST.md"
}