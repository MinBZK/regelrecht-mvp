"""Measure the demo's state at chapters that were written after recording.

    python states_run.py <timeline.json> <viewport.json> <out.json> [url]

The recorder stores the state of the demo with every slide change, and a jump
to a chapter restores it. A chapter added in walkthrough.yaml (`chapters:`
under a take) has no such snapshot: the deck was off when that part was
recorded. This plays the main line on the dev server from the last chapter
that has a snapshot, in real time, and at the start of each added chapter
reads the state the demo is in. Run by
`walkthrough states`, in its own uv environment with Playwright.
"""

import json
import sys

from playwright.sync_api import sync_playwright


def main() -> None:
    timeline_path, viewport_path, out_path = sys.argv[1:4]
    base = sys.argv[4] if len(sys.argv) > 4 else "http://127.0.0.1:7400"
    track = json.load(open(timeline_path))["main"]
    vp = json.load(open(viewport_path))
    found: dict[str, dict] = {}
    with sync_playwright() as p:
        browser = p.chromium.launch(channel="chrome", headless=True, args=["--mute-audio"])
        page = browser.new_context(viewport={"width": vp["width"], "height": vp["height"]}, device_scale_factor=vp.get("dpr", 1)).new_page()
        page.goto(f"{base}/rondleiding")
        page.wait_for_function("() => window.__rrReplay && window.__rrReplay.replay.active", timeout=120000)
        chapters = track["chapters"]
        added = [i for i, c in enumerate(chapters) if c.get("added")]
        # From the last chapter that has a snapshot, in real time: played fast,
        # a request that waits for the engine can be lost, and the state with it.
        start = chapters[max(0, added[0] - 1)]["start"]
        page.evaluate(f"() => window.__rrReplay.seek({start}, {{ play: false }})")
        page.wait_for_function(f"() => Math.abs(window.__rrReplay.replay.now - {start}) < 0.01", timeout=600000)
        marks = [[i, round(chapters[i]["start"] + 0.05, 3)] for i in added]
        page.evaluate(
            """(marks) => {
              window.__states = {};
              const r = window.__rrReplay;
              window.__watch = setInterval(() => {
                // A pause point waits for the viewer; nobody is watching.
                if (r.replay.invite) r.play();
                for (const [i, at] of marks) {
                  if (!(i in window.__states) && r.replay.now >= at) window.__states[i] = r.state();
                }
              }, 40);
              r.play();
            }""",
            marks,
        )
        end = marks[-1][1]
        page.wait_for_function(f"() => Object.keys(window.__states).length === {len(marks)}", timeout=int((end - start) * 2500 + 120000))
        page.evaluate("() => { clearInterval(window.__watch); window.__rrReplay.pause(); }")
        states = page.evaluate("() => window.__states")
        for i in added:
            found[json.dumps(chapters[i]["added"])] = states[str(i)]
            print(f"{chapters[i]['slide'].get('title')}: stand vastgelegd", file=sys.stderr)
        missed = page.evaluate("() => window.__rrReplay.replay.misses.length")
        browser.close()
    if missed:
        raise SystemExit(f"{missed} handelingen vonden hun element niet; de standen kloppen dan niet. Draai eerst `walkthrough verify`.")
    json.dump(found, open(out_path, "w"))


if __name__ == "__main__":
    main()
