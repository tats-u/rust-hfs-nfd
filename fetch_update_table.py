import re
from html.parser import HTMLParser
import requests
from sys import stdout, argv
import json
from collections import deque
from datetime import datetime, timezone
from pathlib import Path


class FetchingDecompositionHTMLParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.in_td = False
        self.in_p = False
        # {"(Unicode char)": "(decomposed chars)"}
        self.encoding_dic = {}
        # {"(element)": {"currnet": "(composed char)", "next": {(sub dictionary)}}}
        self.decoding_dic = {}
        self.overall_regex = re.compile(r"0x([0-9A-F]+)(?: 0x([0-9A-F]+))*")
        self.one_regex = re.compile(r"0x([0-9A-F]+)")
        self.char_to_be_composed = ""

    def handle_starttag(self, tag, attrs):
        """
        Check the beginning of td & p tags
        """
        if tag.lower() == "td":
            self.in_td = True
        if tag.lower() == "p" and self.in_td:
            self.in_p = True

    def handle_endtag(self, tag):
        """
        Check the end of td & p tags
        """
        if tag.lower() == "td" and self.in_td:
            self.in_td = False
        if tag.lower() == "p" and self.in_p:
            self.in_p = False

    def handle_data(self, data):
        if self.in_p and self.in_td:
            overall_match = self.overall_regex.match(data)
            if overall_match is not None:
                codepoints = [
                    chr(int(m[1], 16))
                    for m in (
                        self.one_regex.match(codepoint_str)
                        for codepoint_str in data.split(" ")
                    )
                    if m is not None
                ]
                # decomposition definition
                if len(codepoints) >= 2:
                    self.encoding_dic[self.char_to_be_composed] = "".join(codepoints)
                    self.decoding_dic.setdefault(
                        codepoints[0], {"current": None, "next": {}}
                    )
                    d = self.decoding_dic[codepoints[0]]
                    for c in codepoints[1:]:
                        # `"current": None` may be overwritten later
                        d["next"].setdefault(c, {"current": None, "next": {}})
                        d = d["next"][c]
                    d["current"] = self.char_to_be_composed
                    self.char_to_be_composed = ""
                # character to be decomposed
                else:
                    self.char_to_be_composed = codepoints[0]


if __name__ == "__main__":
    parser = FetchingDecompositionHTMLParser()
    with requests.get(
        "https://developer.apple.com/library/archive/technotes/tn/tn1150table.html"
    ) as req:
        parser.feed(req.text)
    timestamp = datetime.now(timezone.utc).isoformat(timespec="seconds")
    assets_dir = Path(argv[0]).parent / "assets"
    assets_dir.mkdir(exist_ok=True)
    with (assets_dir / "hfs_table.json").open("w", encoding="UTF-8", newline="\n") as f:
        json.dump(
            {
                "created": timestamp,
                "encoding": parser.encoding_dic,
                "decoding": parser.decoding_dic,
            },
            f,
        )

