"""
Generates velotype_sentences.json with 5,000 meaningful records.
- Pulls real English from public-domain sources
- Slices into exact 25-word and 40-word chunks
- Outputs in the exact format your database expects
"""

import json
import random
import re
import urllib.request
from datetime import datetime, timezone

random.seed(2026)

# ---------- SOURCES: real public-domain English ----------
SOURCES = [
    # Project Gutenberg plain-text books (URLs are stable)
    "https://www.gutenberg.org/files/1342/1342-0.txt",   # Pride and Prejudice
    "https://www.gutenberg.org/files/2701/2701-0.txt",   # Moby Dick
    "https://www.gutenberg.org/files/84/84-0.txt",       # Frankenstein
    "https://www.gutenberg.org/files/1661/1661-0.txt",   # Sherlock Holmes
    "https://www.gutenberg.org/files/98/98-0.txt",       # A Tale of Two Cities
    "https://www.gutenberg.org/files/2600/2600-0.txt",   # War and Peace
    "https://www.gutenberg.org/files/1400/1400-0.txt",   # Great Expectations
    "https://www.gutenberg.org/files/345/345-0.txt",     # Dracula
    "https://www.gutenberg.org/files/11/11-0.txt",       # Alice in Wonderland
    "https://www.gutenberg.org/files/74/74-0.txt",       # Tom Sawyer
]

# Fallback text if offline (a few classic public-domain paragraphs)
FALLBACK_TEXT = """
It was the best of times, it was the worst of times, it was the age of wisdom,
it was the age of foolishness, it was the epoch of belief, it was the epoch of
incredulity, it was the season of Light, it was the season of Darkness, it was
the spring of hope, it was the winter of despair. We had everything before us,
we had nothing before us, we were all going direct to Heaven, we were all going
direct the other way. In short, the period was so far like the present period,
that some of its noisiest authorities insisted on its being received, for good
or for evil, in the superlative degree of comparison only. Call me Ishmael.
Some years ago, never mind how long precisely, having little or no money in my
purse, and nothing particular to interest me on shore, I thought I would sail
about a little and see the watery part of the world. It is a way I have of
driving off the spleen and regulating the circulation. Whenever I find myself
growing grim about the mouth; whenever it is a damp, drizzly November in my
soul; whenever I find myself involuntarily pausing before coffin warehouses,
and bringing up the rear of every funeral I meet; and especially whenever my
hypos get such an upper hand of me, that it requires a strong moral principle
to prevent me from deliberately stepping into the street, and methodically
knocking people's hats off, then, I account it high time to get to sea as
soon as I can. This is my substitute for pistol and ball. With a philosophical
flourish Cato throws himself upon his sword; I quietly take to the ship.
There is nothing surprising in this. If they but knew it, almost all men in
their degree, some time or other, cherish very nearly the same feelings
towards the ocean with me. It is a truth universally acknowledged, that a
single man in possession of a good fortune, must be in want of a wife.
However little known the feelings or views of such a man may be on his first
entering a neighbourhood, this truth is so well fixed in the minds of the
surrounding families, that he is considered the rightful property of some one
or other of their daughters. My dear Mr. Bennet, said his lady to him one day,
have you heard that Netherfield Park is let at last? Mr. Bennet replied that
he had not. But it is, returned she; for Mrs. Long has just been here, and she
told me all about it. Mr. Bennet made no answer. Do you not want to know who
has taken it? cried his wife impatiently. You want to tell me, and I have no
objection to hearing it. This was invitation enough.
"""

def fetch_text():
    """Fetch all source texts. Falls back gracefully if offline."""
    corpus = []
    for url in SOURCES:
        try:
            req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
            with urllib.request.urlopen(req, timeout=20) as r:
                raw = r.read().decode("utf-8", errors="ignore")
            corpus.append(clean_gutenberg(raw))
            print(f"✓ fetched: {url}")
        except Exception as e:
            print(f"✗ skipped ({e}): {url}")
    if not corpus:
        print("! using fallback text only")
        corpus.append(FALLBACK_TEXT)
    return "\n\n".join(corpus)

def clean_gutenberg(text):
    """Strip Gutenberg headers/footers and normalize whitespace."""
    text = re.sub(r"\*\*\* ?START OF (THIS|THE) PROJECT GUTENBERG.*?\*\*\*",
                  "", text, flags=re.DOTALL | re.IGNORECASE)
    text = re.sub(r"\*\*\* ?END OF (THIS|THE) PROJECT GUTENBERG.*?\*\*\*",
                  "", text, flags=re.DOTALL | re.IGNORECASE)
    text = re.sub(r"\r\n?", "\n", text)
    text = re.sub(r"[ \t]+", " ", text)
    text = re.sub(r"\n{3,}", "\n\n", text)
    return text.strip()

# ---------- SENTENCE SEGMENTATION ----------
def split_sentences(text):
    """Break text into clean, complete sentences."""
    # Collapse newlines so paragraphs become single lines
    text = re.sub(r"\s+", " ", text)
    # Split on . ! ? followed by space + capital, or end of string
    parts = re.split(r"(?<=[.!?])\s+(?=[A-Z\"'(])", text)
    sentences = []
    for p in parts:
        p = p.strip()
        if not p:
            continue
        if len(p) < 30 or len(p) > 600:
            continue
        if not re.search(r"[.!?]$", p):
            p += "."
        # Must be mostly letters/spaces (avoid tables, page numbers, etc.)
        letters = sum(c.isalpha() or c.isspace() for c in p)
        if letters / max(len(p), 1) < 0.85:
            continue
        sentences.append(p)
    return sentences

# ---------- EXACT WORD-COUNT SLICING ----------
def slice_to_words(sentences, target, needed, used):
    """
    Walk through sentences and greedily merge them until we hit exactly
    `target` words. Emits clean, meaningful chunks.
    """
    chunks = []
    buffer = []
    buf_count = 0

    for s in sentences:
        words = s.split()
        # Skip sentences longer than the target (can't use as-is)
        if len(words) > target:
            continue
        # If adding this sentence overshoots, flush the buffer
        if buf_count + len(words) > target:
            buffer = []
            buf_count = 0
        buffer.append(s)
        buf_count += len(words)
        if buf_count == target:
            chunk = " ".join(buffer)
            chunk = re.sub(r"\s+", " ", chunk).strip()
            if chunk not in used:
                chunks.append(chunk)
                used.add(chunk)
                if len(chunks) >= needed:
                    return chunks
            buffer = []
            buf_count = 0
    return chunks

# ---------- MAIN ----------
def main():
    TOTAL = 5000
    BUCKETS = [
        ("prose",  25, 4500),
        ("quotes", 25, 4500),
        ("prose",  40, 4500),
        ("quotes", 40, 4500),
    ]

    print("Fetching public-domain corpus...")
    raw = fetch_text()
    sentences = split_sentences(raw)
    print(f"✓ extracted {len(sentences)} raw sentences")

    if len(sentences) < 500:
        print("! not enough sentences; using fallback text")
        sentences = split_sentences(FALLBACK_TEXT)

    random.shuffle(sentences)

    used = set()
    records = []
    rid = 1
    now = datetime.now(timezone.utc).isoformat(timespec="seconds")

    for category, wc, amount in BUCKETS:
        pool = sentences[:]  # copy so each bucket can re-slice
        random.shuffle(pool)
        chunks = slice_to_words(pool, wc, amount, used)

        # If not enough, cycle through the corpus again
        safety = 0
        while len(chunks) < amount and safety < 50:
            safety += 1
            random.shuffle(pool)
            extra = slice_to_words(pool, wc, amount - len(chunks), used)
            chunks.extend(extra)

        for text in chunks[:amount]:
            records.append({
                "id": rid,
                "text": text,
                "category": category,
                "word_count": wc,
                "date": now,
            })
            rid += 1
        print(f"✓ {category} {wc}w -> {len(chunks[:amount])} records")

    # Final verification pass
    bad = [r for r in records if len(r["text"].split()) != r["word_count"]]
    print(f"\nVerification: {len(records)} records, {len(bad)} with wrong word count")

    with open("velotype_sentences.json", "w", encoding="utf-8") as f:
        json.dump({"records": records}, f, indent=2, ensure_ascii=False)

    print("📄 saved: velotype_sentences.json")

if __name__ == "__main__":
    main()
