import importlib.util
import unittest
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / "grant-pegasus-tab-bar-permission.py"
SPEC = importlib.util.spec_from_file_location("permission_granter", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
permission_granter = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(permission_granter)


class PermissionCacheTests(unittest.TestCase):
    def test_canonical_node_grants_only_the_two_required_permissions(self):
        self.assertEqual(
            permission_granter.permission_node("/plugin.wasm"),
            '"/plugin.wasm" {\n'
            "    ReadApplicationState\n"
            "    ChangeApplicationState\n"
            "}\n",
        )

    def test_update_replaces_only_this_plugins_entry_and_preserves_unrelated_content(self):
        plugin_path = "/home/serg/.config/zellij/plugins/pegasus-tab-bar.wasm"
        unrelated_before = '"file:other-plugin.wasm" {\n    OpenFiles\n}\n// retained comment\n'
        original = (
            unrelated_before
            + f'"{plugin_path}" {{\n    OpenFiles\n}}\n'
            + '"another-plugin.wasm" {\n    RunCommands\n}\n'
        )

        updated = permission_granter.update_cache(original, plugin_path)

        self.assertTrue(updated.startswith(unrelated_before))
        self.assertTrue(updated.endswith('"another-plugin.wasm" {\n    RunCommands\n}\n'))
        self.assertIn(permission_granter.permission_node(plugin_path), updated)
        self.assertNotIn(f'"{plugin_path}" {{\n    OpenFiles\n}}', updated)


if __name__ == "__main__":
    unittest.main()
