"""Analyze benchmark JSON output: aggregate, per-category, top/bottom performers."""

import json
import sys
from pathlib import Path

CATEGORIES = {
    "E-commerce": ["nike", "amazon", "target", "walmart", "bestbuy", "wayfair",
                   "nordstrom", "etsy", "ebay", "costco", "homedepot", "lowes",
                   "macys", "sephora", "stockx", "shopify"],
    "Real estate": ["zillow", "realtor", "redfin", "trulia"],
    "Jobs": ["indeed", "linkedin", "glassdoor", "ziprecruiter"],
    "News / Media": ["bloomberg", "reuters", "wsj", "nytimes", "cnn", "bbc",
                     "cnbc", "techcrunch", "cloudflare", "theverge", "arstechnica",
                     "wired", "guardian"],
    "Travel": ["booking", "expedia", "airbnb", "hotels", "kayak", "tripadvisor"],
    "Tickets / Events": ["viagogo", "ticketmaster", "stubhub", "eventbrite"],
    "Social": ["instagram", "reddit", "pinterest", "tiktok", "facebook", "twitter",
               "threads", "mastodon"],
    "Docs / Tech": ["docs", "developer", "api", "reference", "github"],
    "Finance / Crypto": ["coingecko", "coinmarketcap", "binance", "robinhood"],
}


def classify(outcome: dict) -> str:
    hay = " ".join(outcome.get("labels", [])) + " " + outcome["name"]
    hay = hay.lower()
    for cat, kws in CATEGORIES.items():
        if any(k in hay for k in kws):
            return cat
    return "Other"


def main(path: str) -> None:
    data = json.loads(Path(path).read_text())
    outcomes = data["outcomes"]
    ok = [o for o in outcomes if o.get("error") is None]
    fail = [o for o in outcomes if o.get("error")]
    readable = [o for o in ok if o.get("readable")]

    print("=== AGGREGATE ===")
    print(f"  total: {data['total_run']}  ok: {data['successes']}  fail: {data['failures']}")
    print(f"  readable: {data['readable_count']}/{data['total_run']} "
          f"({data['readable_count'] * 100 / max(1, data['total_run']):.1f}%)")
    print(f"  avg words: {data['avg_word_count']:.0f}")
    print(f"  avg extract ms: {data['avg_extraction_ms']:.1f}")
    print(f"  label match rate: {data['label_match_rate'] * 100:.1f}%")

    print()
    print("=== BY CATEGORY ===")
    buckets: dict[str, list] = {}
    for o in ok:
        cat = classify(o)
        buckets.setdefault(cat, []).append(o)

    print(f"  {'Category':<20} {'n':>4} {'read':>8} {'avgW':>6} {'avgMs':>6} {'lbl%':>6}")
    for cat, items in sorted(buckets.items(), key=lambda x: -len(x[1])):
        n = len(items)
        r = sum(1 for o in items if o.get("readable"))
        aw = sum(o.get("word_count", 0) for o in items) / n
        am = sum(o.get("extraction_ms", 0) for o in items) / n
        tm = sum(o.get("labels_matched", 0) for o in items)
        tt = sum(o.get("labels_total", 0) for o in items) or 1
        lp = tm / tt * 100
        print(f"  {cat:<20} {n:>4} {r}/{n:<5}  {aw:>5.0f} {am:>6.1f} {lp:>5.1f}%")

    print()
    print("=== TOP 10 word_count ===")
    for o in sorted(ok, key=lambda o: -o.get("word_count", 0))[:10]:
        lm, lt = o.get("labels_matched", 0), o.get("labels_total", 0)
        print(f"  {o['word_count']:>5} words / {o['extraction_ms']:>4}ms  {lm}/{lt}  {o['name']}")

    print()
    print("=== TOP 10 slowest extraction ===")
    for o in sorted(ok, key=lambda o: -o.get("extraction_ms", 0))[:10]:
        print(f"  {o['extraction_ms']:>4}ms / {o['word_count']:>5} words  {o['name']}")

    print()
    print(f"=== NETWORK FAILURES ({len(fail)}) ===")
    for o in fail:
        err = o["error"][:60]
        print(f"  {o['name']:<25} {err}")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "benchmarks/runs/bench-50-cold.json")
