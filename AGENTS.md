This is LRCGET — a desktop music player with synced lyrics support.

Stack: Tauri v2 (Rust backend + Vue 3 frontend).

To help you quickly understand the project architecture:

- For the frontend (Vue), see `src/ARCHITECTURE.md`.
- For the backend (Rust), see `src-tauri/ARCHITECTURE.md`.

Read these files before working on the corresponding frontend or backend tasks.

After completing your changes, update these two `ARCHITECTURE.md` files as needed.

If your changes are related to the backend, run `cargo check` in the `src-tauri/` directory. If your changes are only related to the frontend, you do not need to run any checks.

## Inspecting a running dev build (W3C WebDriver)

Debug builds serve a W3C WebDriver server on `http://127.0.0.1:4445` (via `tauri-plugin-webdriver`, backend details in `src-tauri/ARCHITECTURE.md`); drive it with any WebDriver client (Selenium, WebdriverIO, raw curl). Works on X11 and any Wayland compositor — no extra tools needed.

```bash
setsid npm run tauri dev </dev/null > /tmp/lrcget-dev.log 2>&1 & disown
while ! curl -sf http://127.0.0.1:4445/status >/dev/null; do sleep 5; done
```

```python
from selenium import webdriver
from selenium.webdriver.common.by import By
driver = webdriver.Remote(command_executor="http://127.0.0.1:4445",
                          options=webdriver.ChromeOptions())  # options ignored
driver.get("http://localhost:1420")   # dev-mode URL (Vite); webview is usually already here
driver.find_element(By.CSS_SELECTOR, ".my-button").click()
driver.save_screenshot("/tmp/lrcget.png")          # full webview PNG
print(driver.execute_script("return document.title"))
driver.quit()                                       # always end the session
```
