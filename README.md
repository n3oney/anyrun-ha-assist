# anyrun-ha-assist

An anyrun plugin that lets you use Home Assistant's Assist.

# how 2 build?

`cargo build`

This project depends on sqlite3 to hold a database of queries that are valid and aren't valid for nice fuzzy matching.
It will store entries in `$XDG_CACHE_HOME/anyrun-ha-assist.sqlite3` (defaults to `$HOME/.cache/anyrun-ha-assist.sqlite3`).

# Configuration

This plugin requires a config in your anyrun config directory called `ha_assist.ron`.
The file looks like this:

```ron
Config(
  ha_url: "<URL OF HOME ASSISTANT INSTANCE>",
  ha_token: "<LONG LIVED HA TOKEN>",
  // Optional, defaults to :ha
  prefix: ":ha",
)
```
