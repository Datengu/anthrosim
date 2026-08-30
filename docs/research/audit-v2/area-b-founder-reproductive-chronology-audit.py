#!/usr/bin/env python3
"""Independent arithmetic checker for audit-v2 issue #320."""
DAYS_PER_YEAR = 365

def legacy_parent_valid(parent_birth, child_birth): return parent_birth < child_birth
def female_supported(age, bands): return any(a <= age < b and p > 0 for a,b,p in bands)
def male_supported(age, lo, hi): return lo <= age < hi

def main():
    fertility=[(0,18,0),(18,25,220000),(25,35,250000),(35,40,180000),(40,45,80000),(45,2**32-1,0)]
    child=-100
    assert legacy_parent_valid(child-1, child)
    female_days=[18*365-1,18*365,45*365-1,45*365]
    assert [female_supported(d//365,fertility) for d in female_days] == [False,True,True,False]
    male_days=[18*365-1,18*365,70*365-1,70*365]
    assert [male_supported(d//365,18,70) for d in male_days] == [False,True,True,False]
    custom=[(0,21,0),(21,22,1),(22,2**32-1,0)]
    assert [female_supported(a,custom) for a in (20,21,22)] == [False,True,False]
    print('legacy one-day parent accepted by older-only rule: yes')
    print('female boundary support: reject, accept, accept, reject')
    print('male boundary support: reject, accept, accept, reject')
    print('custom fertility support followed exactly: yes')
if __name__ == '__main__': main()
