from pathlib import Path

p = Path('crates/anthrosim-core/src/population.rs')
s = p.read_text()
old = '''                    if let Some(parent_index_value) = person_index(parent, self.person_count()) {
                        if self.is_alive_index(parent_index_value)
                            && self.households[parent_index_value] == household
                            && let Some(group_index) = assigned_group[parent_index_value]
                            && !parent_groups.contains(&group_index)
                        {
                            parent_groups.push(group_index);
                        }
                    }
'''
new = '''                    if let Some(parent_index_value) = person_index(parent, self.person_count())
                        && self.is_alive_index(parent_index_value)
                        && self.households[parent_index_value] == household
                        && let Some(group_index) = assigned_group[parent_index_value]
                        && !parent_groups.contains(&group_index)
                    {
                        parent_groups.push(group_index);
                    }
'''
if s.count(old) != 1:
    raise SystemExit(f'expected one nested parent-group block, found {s.count(old)}')
p.write_text(s.replace(old, new, 1))
