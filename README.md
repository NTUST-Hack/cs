# cs

NTUST Course Selection Assistant

## Usage

```bash
NTUST Course Selection Assistant

Usage: cs --config <CONFIG_PATH>

Options:
  -c, --config <CONFIG_PATH>  Sets the path to the configuration file
  -h, --help                  Print help
  -V, --version               Print version
```

## Config

### Example:

```toml
[account]
ntustsecret = "YOUR_NTUSTSECRET"

[selection]
# mode = "pre/started/custom"
# custom_select_page_url = "https://courseselection.ntust.edu.tw/AddAndSub/B01/B01"
# custom_select_api_url = "https://courseselection.ntust.edu.tw/AddAndSub/B01/ExtraJoin"
mode = "started"
login_retry_interval = 30000     # 30 secs
session_refresh_interval = 60000 # 60 secs

[query]
threads = 1
semester = "1122"
language = "zh"   # zh/en

[net]
interface.query = "auto"
interface.select = "auto"

[[target.courses]]
course_no = "AT2005701"
enabled = false
force_grab = false

[[target.courses]]
course_no = "TCG036301"
enabled = true
force_grab = false

[[target.courses]]
course_no = "IBG009301"
enabled = true
force_grab = false

[[target.courses]]
course_no = "AT2003301"
enabled = true
force_grab = false

```
