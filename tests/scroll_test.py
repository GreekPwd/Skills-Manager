from playwright.sync_api import sync_playwright


with sync_playwright() as playwright:
    browser = playwright.chromium.launch(channel="chrome", headless=True)
    page = browser.new_page(viewport={"width": 1180, "height": 760})
    page.goto("http://127.0.0.1:1423")
    page.wait_for_load_state("networkidle")

    page.evaluate(
        """
        () => {
          const table = document.querySelector('.skill-table');
          const row = table?.querySelector('.skill-row');
          if (!table || !row) throw new Error('skill table fixture is missing');
          for (let index = 0; index < 80; index += 1) table.append(row.cloneNode(true));
        }
        """
    )

    metrics = page.locator(".skill-table").evaluate(
        """
        (element) => {
          element.scrollTop = element.scrollHeight;
          return {
            clientHeight: element.clientHeight,
            scrollHeight: element.scrollHeight,
            scrollTop: element.scrollTop,
            overflowY: getComputedStyle(element).overflowY,
          };
        }
        """
    )

    assert metrics["overflowY"] == "auto", metrics
    assert metrics["scrollHeight"] > metrics["clientHeight"], metrics
    assert metrics["scrollTop"] > 0, metrics
    browser.close()
