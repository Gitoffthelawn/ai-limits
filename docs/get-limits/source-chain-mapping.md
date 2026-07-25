# Source Chain Interface Mapping

| Interface mode | Source chain |
| --- | --- |
| Terminal default limits output | `fast_free` |
| Terminal `--best` / `-b` | `cli_fallback` |
| Desktop `Fast` | `fast_free` |
| Desktop `Full` | `cli_fallback` |
| Desktop `Best` | `cli_first` |

`--all` is diagnostic: it queries every current source separately and does not apply source chains.
