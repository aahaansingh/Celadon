# Celadon commands

Type commands in the search bar. Commands that take a query show autocomplete suggestions; press **Enter** to run the command or pick the first suggestion.

---

## List views

| Command | Action |
|--------|--------|
| `\f` | Open **All Feeds** |
| `\s` | Open **All Superfeeds** |
| `\t` | Open **All Tags** |
| `\fs` | Open **All Superfeeds** (same as `\s`) |
| `\sf` | Open **All Feeds** (same as `\f`) |

---

## Go to a specific feed, superfeed, or tag

Type the command and a query; pick from the suggestions or press Enter to use the first match.

| Command | Action |
|--------|--------|
| `\f:query` | Go to **Feed** matching the query (articles in that feed) |
| `\s:query` | Go to **Superfeed** matching the query (articles in that superfeed) |
| `\t:query` | Go to **Tag** matching the query (articles with that tag) |

---

## Feeds in a superfeed / Superfeeds for a feed

These open the same views as the right‑click menu options "Show feeds in this superfeed" and "Show superfeeds".

| Command | Action |
|--------|--------|
| `\fs:query` | Search **superfeeds** by name; pick one → open **Feeds in [superfeed]** |
| `\sf:query` | Search **feeds** by name; pick one → open **Superfeeds for [feed]** |

Without a query, `\fs` opens All Superfeeds and `\sf` opens All Feeds.

---

## Read filter (article views only)

When you are on an article view (All, a feed, superfeed, tag, or search), these change which articles are shown:

| Command | Action |
|--------|--------|
| `\a` | Show **All** articles (read + unread) |
| `\r` | Show **Read** only |
| `\u` | Show **Unread** only (default) |

---

## Search with a filter

You can combine a search query with a filter.

**Prefix form**

- `\a:query` — Search all articles (read + unread) for *query*
- `\r:query` — Search read articles for *query*
- `\u:query` — Search unread articles for *query*

**Suffix form**

- `query\a` — Same as `\a:query`
- `query\r` — Same as `\r:query`
- `query\u` — Same as `\u:query`

---

## Plain search

Typing text **without** a leading `\` searches **article titles** and shows suggestions. Press Enter to view matching articles.

---

## Keyboard

- **Enter** — Run the command, or pick the first suggestion when in a search/command with suggestions.
- **Arrow Up / Down** — Change the selected suggestion.
- **Escape** — Clear suggestions.
