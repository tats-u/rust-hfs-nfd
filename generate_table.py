"""
Python script to fetch Unicode decomposition table from https://developer.apple.com/library/archive/technotes/tn/tn1150table.html
and generate Rust source code to compose or decompose Unicode 
"""

import re
from html.parser import HTMLParser
import requests
from sys import stdout
import json
from collections import deque
from datetime import datetime, timezone


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


def print_pre(f=stdout):
    """
    Generate and print the code before the definition of dictionaries
    """
    print(
        f"""\
//! Definition of Unicode decomposition dictionaries
//!
//! Generated based on https://developer.apple.com/library/archive/technotes/tn/tn1150table.html
//! at {datetime.now(timezone.utc).isoformat(timespec="seconds")}

use super::reverse_tree::ReverseTreeNode;
use hashbrown::HashMap;
use lazy_static::lazy_static;
lazy_static! """
        "{",
        file=f,
    )


def print_post(f=stdout):
    """
    Generate and print the code after the definition of dictionaries
    """
    print("}", file=f)


def print_encoding_dic(obj, f=stdout):
    """
    Generate and print the definition of encoding dictionary
    """
    print(
        """\
    /// map from composed character (normal) to decomposed components (HFS+)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// assert_eq!((*MAP_TO_HFS).get(&'\\u{00E9}').unwrap(), "e\\u{0301}");
    /// ```
    pub static ref MAP_TO_HFS: HashMap<char, &'static str> = {
        let mut map = HashMap::new();""",
        file=f,
    )
    for (compose, decompose) in obj.items():
        print(
            f"        map.insert('\\u{{{ord(compose):04X}}}', \""
            + "".join((f"\\u{{{ord(c):04X}}}" for c in decompose))
            + '");',
            file=f,
        )
    print("        return map;\n    };", file=f)


def _print_de_(dic, var_name, f=stdout):
    """
    Body of generator of decoding table

    Use recursion because the depth of recursive calls is limited.
    """
    print(f"        let mut {var_name} = HashMap::new();", file=f)
    for char, result_obj in dic.items():
        char_hexcode = f"{ord(char):04x}"
        current = (
            f"""Some('\\u{{{ord(result_obj["current"]):04X}}}')"""
            if ("current" in result_obj and result_obj["current"])
            else "None"
        )
        if result_obj["next"]:
            new_var_name = (
                f"u{char_hexcode}"
                if var_name == "root"
                else f"{var_name}_{char_hexcode}"
            )
            _print_de_(result_obj["next"], new_var_name, f)
            print(
                f"        {var_name}.insert('\\u{{{char_hexcode.upper()}}}', ReverseTreeNode::new({current}, Some(Box::new({new_var_name}))));",
                file=f,
            )
        else:
            print(
                f"        {var_name}.insert('\\u{{{char_hexcode.upper()}}}', ReverseTreeNode::new({current}, None));",
                file=f,
            )


def print_decoding_dic(dic, f=stdout):
    """
    Generate and print the definition of decoding dictionary
    """
    print(
        """\
    /// Dictionary (map) from decomposed components to sub dictionaries and composed characters
    ///
    /// # Examples
    ///
    /// ```ignore
    /// assert_eq!((*MAP_TO_NORMAL).get(&'e').unwrap().next.unwrap().get(&'\\u{0301}').unwrap().current.unwrap(), '\\u{00E9}');
    /// ```
    pub static ref MAP_TO_NORMAL: HashMap<char, ReverseTreeNode> = {""",
        file=f,
    )
    _print_de_(dic, "root", f)
    print("        return root;\n    };", file=f)


if __name__ == "__main__":
    parser = FetchingDecompositionHTMLParser()
    req = requests.get(
        "https://developer.apple.com/library/archive/technotes/tn/tn1150table.html"
    )
    parser.feed(req.text)
    with open("src/code_table.rs", "w", encoding="utf-8", newline="\n") as f:
        print_pre(f)
        print_encoding_dic(parser.encoding_dic, f)
        print_decoding_dic(parser.decoding_dic, f)
        print_post(f)
