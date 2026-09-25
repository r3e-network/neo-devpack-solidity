import os
import re

mapping = {
    '/devpack/standards': '/additional-material/neo-standards',
    '/devpack/overview': '/additional-material/neo-devpack',
    '/reference/runtime': '/internals/runtime-specification',
    '/reference/parity-limitations': '/internals/parity-and-limitations',
    '/neovm/native-contracts': '/internals/native-contracts',
    '/neovm/syscalls': '/internals/syscalls',
    '/reference/architecture': '/internals/architecture',
    '/reference/errors': '/advisory-content/error-reference',
    '/reference/cli': '/compiler/using-the-compiler',
    '/reference/troubleshooting': '/advisory-content/troubleshooting',
    '/manifests/manifest-spec': '/internals/contract-metadata',
    '/workflows/compile': '/compiler/analysing-the-compiler-output',
    '/workflows/deploy': '/basics/deploying-contracts',
    '/workflows/test': '/basics/testing-contracts',
    '/workflows/production': '/advisory-content/production-readiness',
    '/getting-started/installation': '/basics/installing-the-compiler',
    '/getting-started/quickstart': '/basics/quickstart',
    '/getting-started/overview': '/basics/introduction-to-smart-contracts',
}

def replace_links(file_path):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    original_content = content
    for old_link, new_link in mapping.items():
        # Match both exact strings and strings followed by # anchor links
        pattern = re.compile(rf'({re.escape(old_link)})([#"\)])')
        content = pattern.sub(rf'{new_link}\2', content)

    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"Updated links in {file_path}")

for root, _, files in os.walk('docs'):
    for file in files:
        if file.endswith('.md'):
            replace_links(os.path.join(root, file))
mapping['/language-description/grammar'] = '/language-description/grammar'
mapping['/compiler/ir-codegen-changes'] = '/compiler/ir-codegen-changes'
mapping['/internals/layout-of-call-data'] = '/internals/layout-of-call-data'
mapping['/internals/cleaning-up-variables'] = '/internals/cleaning-up-variables'
mapping['/resources/style-guide'] = '/resources/style-guide'
