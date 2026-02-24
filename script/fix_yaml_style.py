#!/usr/bin/env python3
"""Fix YAML style to comply with yamllint rules."""

import sys
import re
import textwrap
import yaml


def clean_text(text: str) -> str:
    """Clean text to remove problematic characters that break YAML block style."""
    # Remove trailing whitespace from lines
    lines = text.split('\n')
    lines = [line.rstrip() for line in lines]
    text = '\n'.join(lines)
    # Remove any non-printable characters except newlines
    text = ''.join(c for c in text if c == '\n' or (c.isprintable() and ord(c) < 65536))
    return text


def wrap_text(text: str, width: int = 115) -> str:
    """Wrap long lines in text while preserving structure."""
    # First clean the text
    text = clean_text(text)

    lines = text.split('\n')
    wrapped_lines = []

    for line in lines:
        if len(line) > width:
            # Wrap the line
            wrapped = textwrap.fill(
                line,
                width=width,
                break_long_words=True,  # Allow breaking long words
                break_on_hyphens=True
            )
            wrapped_lines.append(wrapped)
        else:
            wrapped_lines.append(line)

    return '\n'.join(wrapped_lines)


class IndentedDumper(yaml.SafeDumper):
    """Custom YAML dumper with proper indentation for sequences."""

    def increase_indent(self, flow=False, indentless=False):
        return super().increase_indent(flow=flow, indentless=False)


def str_representer(dumper, data):
    """Use block style for multiline strings."""
    if '\n' in data:
        # Always use literal block style for multiline
        return dumper.represent_scalar('tag:yaml.org,2002:str', data, style='|')
    # Check if the string needs quoting
    if data in ('true', 'false', 'yes', 'no', 'on', 'off', 'null', '~', ''):
        return dumper.represent_scalar('tag:yaml.org,2002:str', data, style="'")
    # Only quote pure numeric strings, not alphanumeric like "8a"
    if re.match(r'^[\d.+-]+$', data):
        return dumper.represent_scalar('tag:yaml.org,2002:str', data, style="'")
    if data.startswith('$') or data.startswith('#'):
        return dumper.represent_scalar('tag:yaml.org,2002:str', data, style=None)
    return dumper.represent_scalar('tag:yaml.org,2002:str', data, style=None)


IndentedDumper.add_representer(str, str_representer)


def fix_yaml_file(input_path: str, output_path: str = None):
    """Fix YAML file to comply with yamllint rules."""
    if output_path is None:
        output_path = input_path

    # Read the YAML file
    with open(input_path, 'r', encoding='utf-8') as f:
        data = yaml.safe_load(f)

    # Wrap long lines in text fields
    if 'articles' in data:
        for article in data['articles']:
            if 'text' in article and isinstance(article['text'], str):
                article['text'] = wrap_text(article['text'], width=100)

    # Write with proper formatting
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write('---\n')
        yaml.dump(
            data,
            f,
            Dumper=IndentedDumper,
            allow_unicode=True,
            sort_keys=False,
            default_flow_style=False,
            width=120,
            indent=2,
        )


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python fix_yaml_style.py <yaml_file>")
        sys.exit(1)

    input_file = sys.argv[1]
    fix_yaml_file(input_file)
    print(f"Fixed: {input_file}")
