from pathlib import Path

# Production spatial execution must validate against the experiment's own demography.
p = Path('crates/anthrosim-core/src/spatial_simulation.rs')
s = p.read_text()
s = s.replace(
'''                Population::initialize_declared_founder_state_v1(
                    config.population,
                    definition,
                    &world,
                )?''',
'''                Population::initialize_declared_founder_state_v1(
                    config.population,
                    definition,
                    &world,
                    &config.demography,
                )?''')
s = s.replace(
'''            .validate(
                config.population.initial_population,
                config.population.max_person_records,
                world,
            )''',
'''            .validate(
                config.population.initial_population,
                config.population.max_person_records,
                world,
                &config.demography,
            )''')
p.write_text(s)

# Existing isolated fixtures are not research configurations; make their validation schedule
# explicit rather than leaving a weaker 3-argument constructor path.
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
        if not stack and commas == 2:
            insertion = ',\n            &crate::config::DemographyConfig::synthetic_validation_v1()'
            s = s[:i] + insertion + s[i:]
            cursor = i + len(insertion) + 1
            changed = True
        else:
            cursor = i + 1
    if changed:
        p.write_text(s)
