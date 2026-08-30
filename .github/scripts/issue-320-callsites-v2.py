from pathlib import Path

# Migrate every remaining core fixture constructor with an explicit synthetic validation
# schedule. Production Simulation/spatial paths were already bound to config.demography.
token = 'Population::initialize_declared_founder_state_v1('
for p in Path('crates/anthrosim-core/src').rglob('*.rs'):
    s = p.read_text()
    cursor = 0
    changed = False
    while True:
        start = s.find(token, cursor)
        if start < 0:
            break
        open_pos = start + len(token) - 1
        stack = ['(']
        i = open_pos + 1
        commas = 0
        in_string = False
        escape = False
        while i < len(s) and stack:
            ch = s[i]
            if in_string:
                if escape:
                    escape = False
                elif ch == '\\':
                    escape = True
                elif ch == '"':
                    in_string = False
            else:
                if ch == '"':
                    in_string = True
                elif ch in '([{':
                    stack.append(ch)
                elif ch in ')]}':
                    stack.pop()
                    if not stack:
                        break
                elif ch == ',' and len(stack) == 1:
                    commas += 1
            i += 1
        if stack:
            raise RuntimeError(f'unbalanced constructor call in {p}')
        inner = s[open_pos + 1:i]
        arg_count = commas if inner.rstrip().endswith(',') else commas + 1
        if arg_count == 3:
            insertion = ',\n            &crate::config::DemographyConfig::synthetic_validation_v1()'
            s = s[:i] + insertion + s[i:]
            cursor = i + len(insertion) + 1
            changed = True
        else:
            cursor = i + 1
    if changed:
        p.write_text(s)

# CLI reconstruction paths have the complete checkpoint experiment and must use its schedule.
p = Path('crates/anthrosim-cli/src/bundle.rs')
s = p.read_text()
s = s.replace(
    'Population::initialize_declared_founder_state_v1(config, definition, world)',
    'Population::initialize_declared_founder_state_v1(\n                config,\n                definition,\n                world,\n                &checkpoint.experiment.demography,\n            )',
)
p.write_text(s)

p = Path('crates/anthrosim-cli/src/bin/anthrosim-demography-observability.rs')
s = p.read_text()
s = s.replace(
    'Population::initialize_declared_founder_state_v1(config, definition, world)?',
    'Population::initialize_declared_founder_state_v1(\n                config,\n                definition,\n                world,\n                &checkpoint.experiment.demography,\n            )?',
)
p.write_text(s)
