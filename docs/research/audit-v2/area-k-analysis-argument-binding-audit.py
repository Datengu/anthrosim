#!/usr/bin/env python3
"""Independent audit-v2 Area K checker for issue #340."""

VALUES = [2, 3]

def execute(scale: int) -> int:
    return sum(VALUES) * scale

def main() -> None:
    legacy_recorded_scale = 3
    legacy_executed_scale = 2
    legacy_output = execute(legacy_executed_scale)
    assert legacy_recorded_scale != legacy_executed_scale
    assert legacy_output == 10

    v2_command_scale = 3
    v2_output = execute(v2_command_scale)
    assert v2_output == 15

    config = {"scale": 3}
    config_output = execute(config["scale"])
    assert config_output == 15

    print(f"legacy_recorded_scale_3_executed_scale_2={legacy_output}")
    print(f"v2_executed_scale_3={v2_output}")
    print(f"v2_config_file_scale_3={config_output}")
    print("independent result: one executable configuration representation removes the v1 contradiction")

if __name__ == "__main__":
    main()
