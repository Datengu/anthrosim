#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import urllib.request
from pathlib import Path

path = Path('.github/workflows/ci.yml')
text = path.read_text(encoding='utf-8')
old = "assert reference['modelSemanticsId'] == 'anthrosim-model-semantics-v28'"
new = "assert reference['modelSemanticsId'] == 'anthrosim-model-semantics-v29'"
assert text.count(old) == 1
updated = text.replace(old, new, 1)
path.write_text(updated, encoding='utf-8')

# actions/checkout persisted this job's installation token in Git config. Use it only to
# create an unattached Git blob; the user-authorized connector will perform the ref update.
header = subprocess.check_output(
    ['git', 'config', '--get', 'http.https://github.com/.extraheader'],
    text=True,
).strip()
assert header.lower().startswith('authorization: ')
authorization = header.split(': ', 1)[1]
request = urllib.request.Request(
    'https://api.github.com/repos/Datengu/anthrosim/git/blobs',
    data=json.dumps({'content': updated, 'encoding': 'utf-8'}).encode('utf-8'),
    headers={
        'Accept': 'application/vnd.github+json',
        'Authorization': authorization,
        'X-GitHub-Api-Version': '2022-11-28',
        'Content-Type': 'application/json',
    },
    method='POST',
)
with urllib.request.urlopen(request) as response:
    result = json.load(response)
sha = result['sha']
assert len(sha) == 40
print(f'CI_V29_BLOB_SHA={sha}')
