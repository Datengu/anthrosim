from pathlib import Path

path = Path('.github/workflows/ci.yml')
text = path.read_text()
old = "assert reference['modelSemanticsId'] == 'anthrosim-model-semantics-v13'"
new = "assert reference['modelSemanticsId'] == 'anthrosim-model-semantics-v14'"
if text.count(old) != 1:
    raise SystemExit('expected exactly one active M7.6 v13 semantics assertion')
path.write_text(text.replace(old, new, 1))
