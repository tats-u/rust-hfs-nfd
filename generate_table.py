"""
Python script to fetch Unicode decomposition table from https://developer.apple.com/library/archive/technotes/tn/tn1150table.html
and generate Rust source code to compose or decompose Unicode 
"""

from sys import stdout, argv
import json
from pathlib import Path


def print_pre(timestamp: str, f=stdout):
    """
    Generate and print the code before the definition of dictionaries
    """
    print(
        f"""\
//! Definition of Unicode decomposition dictionaries
//!
//! Generated based on https://developer.apple.com/library/archive/technotes/tn/tn1150table.html
//! fetched at {timestamp}

use super::reverse_tree::ReverseTreeNode;
use ahash::AHashMap;
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
    pub static ref MAP_TO_HFS: AHashMap<char, &'static str> = {
        let mut map = AHashMap::new();""",
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
    print(f"        let mut {var_name} = AHashMap::new();", file=f)
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
    pub static ref MAP_TO_NORMAL: AHashMap<char, ReverseTreeNode> = {""",
        file=f,
    )
    _print_de_(dic, "root", f)
    print("        return root;\n    };", file=f)


if __name__ == "__main__":
    root_dir = Path(argv[0]).parent
    assets_dir = root_dir / "assets"
    src_dir = root_dir / "src"

    with (assets_dir / "hfs_table.json").open("r", encoding="UTF-8", newline="\n") as f:
        hfs_table = json.load(f)
    src_dir.mkdir(exist_ok=True)
    with (src_dir / "code_table.rs").open("w", encoding="utf-8", newline="\n") as f:
        print_pre(hfs_table["created"], f)
        print_encoding_dic(hfs_table["encoding"], f)
        print_decoding_dic(hfs_table["decoding"], f)
        print_post(f)
